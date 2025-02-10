use std::sync::{Arc, Mutex};

use crossbeam_channel::{select_biased, Receiver};
use eframe::egui::Color32;
use wg_2024::controller::DroneEvent;
use wg_2024::packet::{NodeType, Packet};

use super::helper;
use crate::shared_data::SimulationData;

/// loop that will be running in the thread that listens for `DroneEvents`
/// and update the shared data accordingly.
pub fn receiver_loop(
    data_ref: &Arc<Mutex<SimulationData>>,
    rec_client: &Receiver<DroneEvent>,
    rec_kill: &Receiver<()>,
) {
    loop {
        select_biased! {
            recv(rec_kill) -> packet => {
                if packet.is_ok() {
                    return;
                }
            }
            recv(rec_client) -> packet => {
                if let Ok(event) = packet {
                    handle_event(data_ref, &event);
                }
            }
        }
    }
}

/// update shared data based on the event
fn handle_event(data_ref: &Arc<Mutex<SimulationData>>, event: &DroneEvent) {
    match event {
        DroneEvent::PacketSent(p) => {
            handle_packet_sent(data_ref, p);
        }
        DroneEvent::PacketDropped(p) => {
            handle_packet_dropped(data_ref, p);
        }
        DroneEvent::ControllerShortcut(p) => {
            handle_controller_shortcut(data_ref, p);
        }
    }
}

/// update shared data when a packet is dropped
fn handle_packet_dropped(data_ref: &Arc<Mutex<SimulationData>>, p: &Packet) {
    let drone_id = p.routing_header.hops[p.routing_header.hop_index];
    let from_id = p.routing_header.hops[p.routing_header.hop_index - 1];
    let mut data = data_ref.lock().unwrap();

    // add log
    data.add_log(
        drone_id,
        format!("Dropped fragment sent by node #{from_id}"),
        Color32::LIGHT_RED,
    );

    // increment stat
    data.drone_stats
        .get_mut(&drone_id)
        .unwrap()
        .fragments_dropped += 1;

    data.ctx.request_repaint();
}

/// update shared data when a packet is sent
fn handle_packet_sent(data_ref: &Arc<Mutex<SimulationData>>, p: &Packet) {
    helper::handle_packet_sent(NodeType::Drone, p, data_ref);
}

/// update shared data when a packet is sent to the simulation controller
fn handle_controller_shortcut(data_ref: &Arc<Mutex<SimulationData>>, p: &Packet) {
    let data = data_ref.lock().unwrap();
    _ = data.sc.shortcut(p.clone());
}
