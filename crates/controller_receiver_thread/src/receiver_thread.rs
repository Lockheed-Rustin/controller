use std::sync::{Arc, Mutex};

use crossbeam_channel::{select, Receiver};

use controller_data::SimulationData;
use wg_2024::controller::NodeEvent;

pub fn receiver_loop(data_ref: Arc<Mutex<SimulationData>>, rec: Receiver<NodeEvent>) {
    loop {
        select! {
            recv(rec) -> packet => {
                if let Ok(packet) = packet {
                    match packet {
                        NodeEvent::PacketSent(_) => {
                            println!("PacketSent");
                        }
                        NodeEvent::PacketDropped(_) => {
                            println!("PacketDropped");
                        }
                        NodeEvent::ControllerShortcut(_) => {
                            println!("ControllerShortcut");
                        }
                    }
                }
            }
        }
    }
}
