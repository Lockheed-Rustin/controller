use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crossbeam_channel::{select, Receiver, Sender};

use wg_2024::controller::DroneEvent;
use wg_2024::network::NodeId;
use wg_2024::packet::{Packet, PacketType};

use controller_data::SimulationData;

use crate::helper;
use crate::helper::get_packet_type_str;

pub fn drone_receiver_loop(
    data_ref: Arc<Mutex<SimulationData>>,
    rec: Receiver<DroneEvent>,
    packet_senders: HashMap<NodeId, Sender<Packet>>,
) {
    loop {
        select! {
            recv(rec) -> packet => {
                if let Ok(event) = packet {
                    handle_event(Arc::clone(&data_ref), event, &packet_senders);
                }
            }
        }
    }
}

fn handle_event(
    data_ref: Arc<Mutex<SimulationData>>,
    event: DroneEvent,
    packet_senders: &HashMap<NodeId, Sender<Packet>>,
) {
    match event {
        DroneEvent::PacketSent(p) => {
            handle_packet_sent(data_ref, &p);
        }
        DroneEvent::PacketDropped(p) => {
            handle_packet_dropped(data_ref, &p);
        }
        DroneEvent::ControllerShortcut(p) => {
            handle_controller_shortcut(data_ref, p, packet_senders);
        }
    }
}

fn handle_packet_dropped(data_ref: Arc<Mutex<SimulationData>>, p: &Packet) {
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

fn handle_packet_sent(data_ref: Arc<Mutex<SimulationData>>, p: &Packet) {
    let (from_id, to_id) = helper::get_from_and_to_packet_send(p);
    let log_line = helper::get_log_line_packet_sent(p, to_id);
    update_data_packet_sent(data_ref, &p.pack_type, &from_id, log_line);
}

fn handle_controller_shortcut(
    data_ref: Arc<Mutex<SimulationData>>,
    p: Packet,
    packet_senders: &HashMap<NodeId, Sender<Packet>>,
) {
    let dest_id = if let Some(&i) = p.routing_header.hops.last() {
        i
    } else {
        return;
    };
    let sender = if let Some(s) = packet_senders.get(&dest_id) {
        s
    } else {
        return;
    };

    match sender.send(p.clone()) {
        Ok(_) => {
            let drone_id = &p.routing_header.hops[p.routing_header.hop_index];
            let log_line = format!(
                "{} sent to Simulation Controller",
                get_packet_type_str(&p.pack_type)
            );
            update_data_packet_sent(data_ref, &p.pack_type, drone_id, log_line)
        }
        Err(_) => {}
    }
}

fn update_data_packet_sent(
    data_ref: Arc<Mutex<SimulationData>>,
    pt: &PacketType,
    id: &NodeId,
    log_line: String,
) {
    let stat_index = helper::get_packet_stat_index(pt);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(id).unwrap().push(log_line);
    data.stats.get_mut(id).unwrap().packets_forwarded[stat_index] += 1;

    data.ctx.request_repaint();
}
