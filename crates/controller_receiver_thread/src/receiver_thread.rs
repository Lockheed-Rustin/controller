use std::sync::{Arc, Mutex};

use crossbeam_channel::{select, Receiver};

use wg_2024::controller::DroneEvent;
use wg_2024::network::NodeId;
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
    let drone_id = get_drone_id(&p);
    let mut data = data_ref.lock().unwrap();

    // add log
    data.logs
        .get_mut(&drone_id)
        .unwrap()
        .push("Packet sent!".to_string());

    // increment stat
    let index = match p.pack_type {
        PacketType::MsgFragment(_) => 0,
        PacketType::Ack(_) => 1,
        PacketType::Nack(_) => 2,
        PacketType::FloodRequest(_) => 3,
        PacketType::FloodResponse(_) => 4,
    };
    data.stats.get_mut(&drone_id).unwrap().packets_forwarded[index] += 1;

    data.ctx.request_repaint();
}

fn handle_packet_dropped(data_ref: Arc<Mutex<SimulationData>>, p: Packet) {
    let drone_id = get_drone_id(&p);
    let mut data = data_ref.lock().unwrap();

    // add log
    data.logs
        .get_mut(&drone_id)
        .unwrap()
        .push("Packet dropped!".to_string());

    // increment stat
    data.stats.get_mut(&drone_id).unwrap().fragments_dropped += 1;

    data.ctx.request_repaint();
}

fn get_drone_id(p: &Packet) -> NodeId {
    p.routing_header.hops[p.routing_header.hop_index - 1]
}
