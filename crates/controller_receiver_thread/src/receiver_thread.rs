use std::sync::{Arc, Mutex};

use crossbeam_channel::{select, Receiver};

use wg_2024::controller::DroneEvent;
use wg_2024::packet::{Packet, PacketType};

use controller_data::SimulationData;

pub fn receiver_loop(data_ref: Arc<Mutex<SimulationData>>, rec: Receiver<DroneEvent>) {
    loop {
        select! {
            recv(rec) -> packet => {
                if let Ok(event) = packet {
                    handle_drone_event(Arc::clone(&data_ref), event);
                }
            }
        }
    }
}

fn handle_drone_event(data_ref: Arc<Mutex<SimulationData>>, event: DroneEvent) {
    match event {
        DroneEvent::PacketSent(p) => {
            handle_packet_sent(data_ref, p);
        }
        DroneEvent::PacketDropped(p) => {
            handle_packet_dropped(data_ref, p);
        }
        DroneEvent::ControllerShortcut(_) => {
            println!("ControllerShortcut");
        }
    }
}

fn handle_packet_sent(data_ref: Arc<Mutex<SimulationData>>, p: Packet) {
    // get sender and receiver ids
    let from_id = if let PacketType::FloodRequest(fr) = &p.pack_type {
        fr.path_trace.last().unwrap().0
    } else {
        p.routing_header.hops[p.routing_header.hop_index - 1]
    };
    let to_id = if let PacketType::FloodRequest(fr) = &p.pack_type {
        None
    } else {
        Some(p.routing_header.hops[p.routing_header.hop_index])
    };

    let mut data = data_ref.lock().unwrap();

    // add log
    let log_line = match to_id {
        None => format!("{} broadcasted", get_packet_type_str(&p.pack_type)),
        Some(id) => format!("{} sent to node #{}", get_packet_type_str(&p.pack_type), id),
    };
    data.logs.get_mut(&from_id).unwrap().push(log_line);

    // increment stat
    let index = match p.pack_type {
        PacketType::MsgFragment(_) => 0,
        PacketType::Ack(_) => 1,
        PacketType::Nack(_) => 2,
        PacketType::FloodRequest(_) => 3,
        PacketType::FloodResponse(_) => 4,
    };
    data.stats.get_mut(&from_id).unwrap().packets_forwarded[index] += 1;

    data.ctx.request_repaint();
}

fn handle_packet_dropped(data_ref: Arc<Mutex<SimulationData>>, p: Packet) {
    let drone_id = p.routing_header.hops[p.routing_header.hop_index];
    let from_id = p.routing_header.hops[p.routing_header.hop_index - 1];
    let mut data = data_ref.lock().unwrap();

    // add log
    data.logs
        .get_mut(&drone_id)
        .unwrap()
        .push(format!("Dropped fragment sent by node #{}", from_id));

    // increment stat
    data.stats.get_mut(&drone_id).unwrap().fragments_dropped += 1;

    data.ctx.request_repaint();
}

fn get_packet_type_str(t: &PacketType) -> &'static str {
    match t {
        PacketType::MsgFragment(_) => "Fragment",
        PacketType::Ack(_) => "Ack",
        PacketType::Nack(_) => "Nack",
        PacketType::FloodRequest(_) => "Flood request",
        PacketType::FloodResponse(_) => "Flood response",
    }
}
