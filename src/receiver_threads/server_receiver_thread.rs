use std::sync::{Arc, Mutex};

use crossbeam_channel::{select_biased, Receiver};

use drone_network::controller::ServerEvent;
use drone_network::message::{ClientBody, ServerBody};
use eframe::egui::Color32;
use wg_2024::network::NodeId;
use wg_2024::packet::{NodeType, Packet};

use super::helper;
use crate::shared_data::SimulationData;

/// loop that will be running in the thread that listens for ServerEvents
/// and update the shared data accordingly
pub fn receiver_loop(
    data_ref: &Arc<Mutex<SimulationData>>,
    rec_client: &Receiver<ServerEvent>,
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
fn handle_event(data_ref: &Arc<Mutex<SimulationData>>, event: &ServerEvent) {
    match event {
        ServerEvent::PacketSent(p) => handle_packet_sent(data_ref, p),
        ServerEvent::PacketReceived(p, id) => handle_packet_received(data_ref, p, *id),
        ServerEvent::MessageAssembled { body, from, to } => {
            handle_message_assembled(data_ref, body, *from, *to);
        }
        ServerEvent::MessageFragmented { body, from, to } => {
            handle_message_fragmented(data_ref, body, *from, *to);
        }
    }
}

/// update shared data when a packet is sent
fn handle_packet_sent(data_ref: &Arc<Mutex<SimulationData>>, p: &Packet) {
    helper::handle_packet_sent(NodeType::Server, p, data_ref);
}

/// update shared data when a packet is received
fn handle_packet_received(data_ref: &Arc<Mutex<SimulationData>>, p: &Packet, id: NodeId) {
    helper::handle_packet_received(id, NodeType::Server, p, data_ref);
}

/// update shared data when a message is assembled
fn handle_message_assembled(
    data_ref: &Arc<Mutex<SimulationData>>,
    body: &ClientBody,
    from: NodeId,
    to: NodeId,
) {
    let mut log_line = format!("Assembled message from client #{from}\n");
    log_line.push_str(&helper::get_log_line_client_body(body));
    let mut data = data_ref.lock().unwrap();
    data.add_log(to, log_line, Color32::WHITE);
    data.server_stats.get_mut(&to).unwrap().messages_assembled += 1;
    data.ctx.request_repaint();
}

/// update shared data when a message is fragmented
fn handle_message_fragmented(
    data_ref: &Arc<Mutex<SimulationData>>,
    body: &ServerBody,
    from: NodeId,
    to: NodeId,
) {
    let mut log_line = format!("Fragmented message for client #{to}\n");
    log_line.push_str(&helper::get_log_line_server_body(body));
    let mut data = data_ref.lock().unwrap();
    data.add_log(from, log_line, Color32::WHITE);
    data.server_stats
        .get_mut(&from)
        .unwrap()
        .messages_fragmented += 1;
    data.ctx.request_repaint();
}
