mod util;

use std::sync::{Arc, Mutex};

use crossbeam_channel::{select, Receiver};

use wg_2024::controller::DroneEvent;
use wg_2024::packet::Packet;

use controller_data::SimulationData;
use drone_networks::controller::{ClientEvent, ServerEvent};
use wg_2024::network::NodeId;

// ----- Drone -----
pub fn drone_receiver_loop(data_ref: Arc<Mutex<SimulationData>>, rec: Receiver<DroneEvent>) {
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
            handle_drone_packet_sent(data_ref, p);
        }
        DroneEvent::PacketDropped(p) => {
            handle_drone_packet_dropped(data_ref, p);
        }
        DroneEvent::ControllerShortcut(_) => {
            println!("ControllerShortcut");
        }
    }
}

fn handle_drone_packet_dropped(data_ref: Arc<Mutex<SimulationData>>, p: Packet) {
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

fn handle_drone_packet_sent(data_ref: Arc<Mutex<SimulationData>>, p: Packet) {
    let (from_id, to_id) = util::get_from_and_to_packet_send(&p);
    let log_line = util::get_log_line_packet_sent(&p, to_id);

    let stat_index = util::get_packet_stat_index(&p.pack_type);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&from_id).unwrap().push(log_line);
    data.stats.get_mut(&from_id).unwrap().packets_forwarded[stat_index] += 1;

    data.ctx.request_repaint();
}

// ----- Client -----
pub fn client_receiver_loop(data_ref: Arc<Mutex<SimulationData>>, rec: Receiver<ClientEvent>) {
    loop {
        select! {
            recv(rec) -> packet => {
                if let Ok(event) = packet {
                    handle_client_event(Arc::clone(&data_ref), event);
                }
            }
        }
    }
}

fn handle_client_event(data_ref: Arc<Mutex<SimulationData>>, event: ClientEvent) {
    match event {
        ClientEvent::PacketReceived(p, id) => handle_client_packet_received(data_ref, p, id),
        ClientEvent::MessageAssembled(_) => {}
        ClientEvent::MessageFragmented(_) => {}
        ClientEvent::PacketSent(p) => handle_client_packet_sent(data_ref, p),
    }
}

fn handle_client_packet_sent(data_ref: Arc<Mutex<SimulationData>>, p: Packet) {
    let (from_id, to_id) = util::get_from_and_to_packet_send(&p);
    let log_line = util::get_log_line_packet_sent(&p, to_id);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&from_id).unwrap().push(log_line);
    // update client stats
    data.ctx.request_repaint();
}

fn handle_client_packet_received(data_ref: Arc<Mutex<SimulationData>>, p: Packet, id: NodeId) {
    let from_id = util::get_from_packet_received(&p);
    let log_line = util::get_log_line_packet_received(&p, from_id);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&id).unwrap().push(log_line);
    // update client stats
    data.ctx.request_repaint();
}

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
    let (from_id, to_id) = util::get_from_and_to_packet_send(&p);
    let log_line = util::get_log_line_packet_sent(&p, to_id);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&from_id).unwrap().push(log_line);
    // update server stats
    data.ctx.request_repaint();
}

// copy of client except updating stats
fn handle_server_packet_received(data_ref: Arc<Mutex<SimulationData>>, p: Packet, id: NodeId) {
    let from_id = util::get_from_packet_received(&p);
    let log_line = util::get_log_line_packet_received(&p, from_id);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&id).unwrap().push(log_line);
    // update server stats
    data.ctx.request_repaint();
}
