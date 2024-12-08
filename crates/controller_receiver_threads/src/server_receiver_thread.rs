use std::sync::{Arc, Mutex};

use crossbeam_channel::{select, Receiver};

use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

use drone_networks::controller::ServerEvent;

use controller_data::SimulationData;

use crate::helper;

// ----- Server -----
pub fn server_receiver_loop(data_ref: Arc<Mutex<SimulationData>>, rec: Receiver<ServerEvent>) {
    loop {
        select! {
            recv(rec) -> packet => {
                if let Ok(event) = packet {
                    handle_server_event(Arc::clone(&data_ref), event);
                }
            }
        }
    }
}

fn handle_server_event(data_ref: Arc<Mutex<SimulationData>>, event: ServerEvent) {
    match event {
        ServerEvent::PacketReceived(p, id) => handle_server_packet_received(data_ref, p, id),
        ServerEvent::MessageAssembled(_) => {}
        ServerEvent::MessageFragmented(_) => {}
        ServerEvent::PacketSent(p) => handle_server_packet_sent(data_ref, p),
    }
}

// copy of client except updating stats
fn handle_server_packet_sent(data_ref: Arc<Mutex<SimulationData>>, p: Packet) {
    let (from_id, to_id) = helper::get_from_and_to_packet_send(&p);
    let log_line = helper::get_log_line_packet_sent(&p, to_id);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&from_id).unwrap().push(log_line);
    // update server stats
    data.ctx.request_repaint();
}

// copy of client except updating stats
fn handle_server_packet_received(data_ref: Arc<Mutex<SimulationData>>, p: Packet, id: NodeId) {
    let from_id = helper::get_from_packet_received(&p);
    let log_line = helper::get_log_line_packet_received(&p, from_id);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&id).unwrap().push(log_line);
    // update server stats
    data.ctx.request_repaint();
}
