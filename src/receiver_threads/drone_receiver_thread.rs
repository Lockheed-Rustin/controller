use std::sync::{Arc, Mutex};

use crossbeam_channel::{select_biased, Receiver};
use eframe::egui::Color32;
use wg_2024::controller::DroneEvent;
use wg_2024::packet::{NodeType, Packet};

use super::helper;
use crate::shared_data::SimulationData;

pub fn receiver_loop(
    data_ref: Arc<Mutex<SimulationData>>,
    rec_client: Receiver<DroneEvent>,
    rec_kill: Receiver<()>,
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
                    handle_event(&data_ref, event);
                }
            }
        }
    }
}

fn handle_event(data_ref: &Arc<Mutex<SimulationData>>, event: DroneEvent) {
    match event {
        DroneEvent::PacketSent(p) => {
            handle_packet_sent(data_ref, &p);
        }
        DroneEvent::PacketDropped(p) => {
            handle_packet_dropped(data_ref, &p);
        }
        DroneEvent::ControllerShortcut(p) => {
            handle_controller_shortcut(data_ref, &p);
        }
    }
}

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

fn handle_packet_sent(data_ref: &Arc<Mutex<SimulationData>>, p: &Packet) {
    helper::handle_packet_sent(NodeType::Drone, p, data_ref);
}

fn handle_controller_shortcut(data_ref: &Arc<Mutex<SimulationData>>, p: &Packet) {
    // TODO: check who really sent it
    // let from_id = p.routing_header.hops[p.routing_header.hop_index];
    // let to_id = *p.routing_header.hops.last().unwrap();
    // let log_line = format!(
    //     "{} sent to Simulation Controller, recipient: node #{}",
    //     get_packet_type_str(&p.pack_type),
    //     to_id
    // );
    // let stat_index = helper::get_packet_stat_index(&p.pack_type);

    let data = data_ref.lock().unwrap();
    _ = data.sc.shortcut(p.clone());
    // if data.sc.shortcut(p.clone()).is_some() {
    //     data.logs.get_mut(&from_id).unwrap().push(log_line);
    //     data.drone_stats
    //         .get_mut(&from_id)
    //         .unwrap()
    //         .packets_forwarded[stat_index] += 1;
    //
    //     data.ctx.request_repaint();
    // }
}
