use std::sync::{Arc, Mutex};

use crossbeam_channel::{select_biased, Receiver};

use drone_networks::controller::ServerEvent;
use drone_networks::message::{ClientBody, ServerBody};
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

use super::helper;
use crate::data::SimulationData;
use crate::receiver_threads::helper::{get_log_line_client_body, get_log_line_server_body};

// ----- Server -----
pub fn receiver_loop(
    data_ref: Arc<Mutex<SimulationData>>,
    rec_client: Receiver<ServerEvent>,
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
                    handle_event(Arc::clone(&data_ref), event);
                }
            }
        }
    }
}

fn handle_event(data_ref: Arc<Mutex<SimulationData>>, event: ServerEvent) {
    match event {
        ServerEvent::PacketSent(p) => handle_packet_sent(data_ref, p),
        ServerEvent::PacketReceived(p, id) => handle_packet_received(data_ref, p, id),
        ServerEvent::MessageAssembled { body, from, to } => {
            handle_message_assembled(data_ref, body, from, to);
        }
        ServerEvent::MessageFragmented { body, from, to } => {
            handle_message_fragmented(data_ref, body, from, to);
        }
    }
}

// copy of client except updating stats
fn handle_packet_sent(data_ref: Arc<Mutex<SimulationData>>, p: Packet) {
    let (from_id, to_id) = helper::get_from_and_to_packet_send(&p);
    let log_line = helper::get_log_line_packet_sent(&p, to_id);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&from_id).unwrap().push(log_line);
    // update server stats
    data.ctx.request_repaint();
}

// copy of client except updating stats
fn handle_packet_received(data_ref: Arc<Mutex<SimulationData>>, p: Packet, id: NodeId) {
    let from_id = helper::get_from_packet_received(&p);
    let log_line = helper::get_log_line_packet_received(&p, from_id);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&id).unwrap().push(log_line);
    // update server stats
    data.ctx.request_repaint();
}

fn handle_message_assembled(
    data_ref: Arc<Mutex<SimulationData>>,
    body: ClientBody,
    from: NodeId,
    to: NodeId,
) {
    let mut log_line = format!("Assembled message from node #{}\n", from);
    log_line.push_str(&get_log_line_client_body(body));
    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&to).unwrap().push(log_line);
    data.client_stats.get_mut(&to).unwrap().messages_assembled += 1;
    data.ctx.request_repaint();
}

fn handle_message_fragmented(
    data_ref: Arc<Mutex<SimulationData>>,
    body: ServerBody,
    from: NodeId,
    to: NodeId,
) {
    let mut log_line = format!("Fragmented message for node #{}", to);
    log_line.push_str(&get_log_line_server_body(body));
    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&from).unwrap().push(log_line);
    data.client_stats
        .get_mut(&from)
        .unwrap()
        .messages_fragmented += 1;
    data.ctx.request_repaint();
}
