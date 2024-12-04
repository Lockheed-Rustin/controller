use std::sync::{Arc, Mutex};

use crossbeam_channel::{select, Receiver};

use controller_data::SimulationData;
use wg_2024::controller::DroneEvent;
use wg_2024::packet::{Packet, PacketType};

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
        DroneEvent::PacketDropped(_) => {
            println!("PacketDropped");
        }
        DroneEvent::ControllerShortcut(_) => {
            println!("ControllerShortcut");
        }
    }
}

fn handle_packet_sent(data_ref: Arc<Mutex<SimulationData>>, p: Packet) {
    let drone_id = p.routing_header.hops[p.routing_header.hop_index - 1];
    let mut data = data_ref.lock().unwrap();

    // add log
    data.logs.get_mut(&drone_id).unwrap()
        .push("Packet sent!".to_string());

    // increment stat
    let index = match p.pack_type {
        PacketType::MsgFragment(_) => 0,
        PacketType::Nack(_) => 1,
        PacketType::Ack(_) => 2,
        PacketType::FloodRequest(_) => 3,
        PacketType::FloodResponse(_) => 4,
    };
    data.stats.get_mut(&drone_id).unwrap().packets_forwarded[index] += 1;

    data.ctx.request_repaint();
}
