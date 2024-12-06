use std::sync::{Arc, Mutex};
use controller_data::SimulationData;
use crossbeam_channel::{select, Receiver};
use drone_networks::controller::ServerEvent;
use wg_2024::network::NodeId;
use wg_2024::packet::{Packet, PacketType};

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
        ServerEvent::PacketReceived(p, id) => handle_packet_received(data_ref, p, id),
        ServerEvent::MessageAssembled(_) => {}
        ServerEvent::MessageFragmented(_) => {}
        ServerEvent::PacketSent(p) => handle_packet_sent(data_ref, p),
    }
}

fn get_packet_type_str(t: &PacketType) -> &'static str {
    match t {
        PacketType::MsgFragment(_) => "Fragment",
        PacketType::Ack(_) => "Ack",
        PacketType::Nack(_) => "Nack",
        PacketType::FloodRequest(_) => "Flood request",
        PacketType::FloodResponse(_) => "Flood response",
    }
}


fn handle_packet_sent(data_ref: Arc<Mutex<SimulationData>>, p: Packet) {
    // get sender and receiver ids
    let from_id = if let PacketType::FloodRequest(fr) = &p.pack_type {
        fr.path_trace.last().unwrap().0
    } else {
        p.routing_header.hops[p.routing_header.hop_index - 1]
    };
    let to_id = if let PacketType::FloodRequest(fr) = &p.pack_type {
        None
    } else {
        Some(p.routing_header.hops[p.routing_header.hop_index])
    };

    let mut data = data_ref.lock().unwrap();

    // add log
    let log_line = match to_id {
        None => format!("{} broadcasted", get_packet_type_str(&p.pack_type)),
        Some(id) => format!("{} sent to node #{}", get_packet_type_str(&p.pack_type), id),
    };
    data.logs.get_mut(&from_id).unwrap().push(log_line);

    // increment stat
    // let index = match p.pack_type {
    //     PacketType::MsgFragment(_) => 0,
    //     PacketType::Ack(_) => 1,
    //     PacketType::Nack(_) => 2,
    //     PacketType::FloodRequest(_) => 3,
    //     PacketType::FloodResponse(_) => 4,
    // };
    // data.stats.get_mut(&from_id).unwrap().packets_forwarded[index] += 1;

    data.ctx.request_repaint();
}


fn handle_packet_received(data_ref: Arc<Mutex<SimulationData>>, p: Packet, to_id: NodeId) {
    // get sender and receiver ids
    let from_id = if let PacketType::FloodRequest(fr) = &p.pack_type {
        fr.path_trace.last().unwrap().0
    } else {
        p.routing_header.hops[p.routing_header.hop_index - 1]
    };

    let mut data = data_ref.lock().unwrap();

    // add log
    let log_line = format!("Received {} from node #{}", get_packet_type_str(&p.pack_type), from_id);
    data.logs.get_mut(&to_id).unwrap().push(log_line);

    // increment stat
    // let index = match p.pack_type {
    //     PacketType::MsgFragment(_) => 0,
    //     PacketType::Ack(_) => 1,
    //     PacketType::Nack(_) => 2,
    //     PacketType::FloodRequest(_) => 3,
    //     PacketType::FloodResponse(_) => 4,
    // };
    // data.stats.get_mut(&from_id).unwrap().packets_forwarded[index] += 1;

    data.ctx.request_repaint();
}