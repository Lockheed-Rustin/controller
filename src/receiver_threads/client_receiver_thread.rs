use std::sync::{Arc, Mutex};

use crossbeam_channel::{select, select_biased, Receiver};

use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

use drone_networks::controller::ClientEvent;

use super::helper;
use crate::data::SimulationData;

// ----- Client -----
pub fn receiver_loop(data_ref: Arc<Mutex<SimulationData>>, rec_client: Receiver<ClientEvent>, rec_kill: Receiver<()>) {
    loop {
        select_biased! {
            recv(rec_kill) -> packet => {
                if let Ok(_) = packet {
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

fn handle_event(data_ref: Arc<Mutex<SimulationData>>, event: ClientEvent) {
    match event {
        ClientEvent::PacketReceived(p, id) => handle_packet_received(data_ref, p, id),
        ClientEvent::MessageAssembled(_) => {}
        ClientEvent::MessageFragmented(_) => {}
        ClientEvent::PacketSent(p) => handle_packet_sent(data_ref, p),
    }
}

fn handle_packet_sent(data_ref: Arc<Mutex<SimulationData>>, p: Packet) {
    let (from_id, to_id) = helper::get_from_and_to_packet_send(&p);
    let log_line = helper::get_log_line_packet_sent(&p, to_id);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&from_id).unwrap().push(log_line);
    // update client stats
    data.ctx.request_repaint();
}

fn handle_packet_received(data_ref: Arc<Mutex<SimulationData>>, p: Packet, id: NodeId) {
    let from_id = helper::get_from_packet_received(&p);
    let log_line = helper::get_log_line_packet_received(&p, from_id);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&id).unwrap().push(log_line);
    // update client stats
    data.ctx.request_repaint();
}
