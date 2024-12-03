use std::sync::{Arc, Mutex};

use crossbeam_channel::{select, Receiver};

use controller_data::SimulationData;
use eframe::egui::TextBuffer;
use wg_2024::controller::DroneEvent;

pub fn receiver_loop(data_ref: Arc<Mutex<SimulationData>>, rec: Receiver<DroneEvent>) {
    loop {
        select! {
            recv(rec) -> packet => {
                if let Ok(packet) = packet {
                    match packet {
                        DroneEvent::PacketSent(p) => {

                            let drone_id = p.routing_header.hops[p.routing_header.hop_index - 1];

                            data_ref.lock().unwrap().logs.get_mut(&drone_id).unwrap()
                                .push_str("\nPacket sent!".as_str());
                            data_ref.lock().unwrap().ctx.request_repaint();

                        }
                        DroneEvent::PacketDropped(_) => {
                            println!("PacketDropped");
                        }
                        DroneEvent::ControllerShortcut(_) => {
                            println!("ControllerShortcut");
                        }
                    }
                }
            }
        }
    }
}
