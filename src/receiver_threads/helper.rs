use crate::data::SimulationData;
use crate::receiver_threads::helper;
use drone_networks::message::{
    ClientBody, ClientCommunicationBody, ClientContentBody, ServerBody, ServerCommunicationBody,
    ServerContentBody,
};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use wg_2024::network::NodeId;
use wg_2024::packet::{NodeType, Packet, PacketType};

// all nodes -----
pub fn handle_packet_sent(sender_type: NodeType, p: &Packet, data_ref: Arc<Mutex<SimulationData>>) {
    let (from_id, to_id) = get_from_and_to_packet_send(p);
    let log_line = get_log_line_packet_sent(p, to_id);
    let stat_index = get_packet_stat_index(&p.pack_type);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&from_id).unwrap().push(log_line);
    match sender_type {
        NodeType::Client => {
            data.client_stats.get_mut(&from_id).unwrap().packets_sent[stat_index] += 1;
        }
        NodeType::Drone => {
            data.drone_stats
                .get_mut(&from_id)
                .unwrap()
                .packets_forwarded[stat_index] += 1;
        }
        NodeType::Server => {
            // TODO: server stats
            // data.server_stats.get_mut(&from_id).unwrap().packets_sent[stat_index] += 1;
        }
    }
    data.ctx.request_repaint();
}


fn get_log_line_packet_sent(p: &Packet, to_id: Option<NodeId>) -> String {
    match to_id {
        None => format!("{} broadcasted", get_packet_type_str(&p.pack_type)),
        Some(id) => match &p.pack_type {
            PacketType::FloodResponse(f) => {
                format!(
                    "{} sent to node #{}\npath trace = {}",
                    get_packet_type_str(&p.pack_type),
                    id,
                    format_path_trace(&f.path_trace),
                )
            }
            _ => format!("{} sent to node #{}", get_packet_type_str(&p.pack_type), id),
        },
    }
}

fn get_from_and_to_packet_send(p: &Packet) -> (NodeId, Option<NodeId>) {
    let from_id = if let PacketType::FloodRequest(fr) = &p.pack_type {
        fr.path_trace.last().unwrap().0
    } else {
        p.routing_header.hops[p.routing_header.hop_index - 1]
    };
    let to_id = if let PacketType::FloodRequest(_) = &p.pack_type {
        None
    } else {
        Some(p.routing_header.hops[p.routing_header.hop_index])
    };
    (from_id, to_id)
}

// clients and servers -----
pub fn handle_packet_received(
    receiver_id: NodeId,
    receiver_type: NodeType,
    p: &Packet,
    data_ref: Arc<Mutex<SimulationData>>,
) {
    let log_line = get_log_line_packet_received(&p, receiver_id);
    let stat_index = get_packet_stat_index(&p.pack_type);

    let mut data = data_ref.lock().unwrap();
    data.logs.get_mut(&receiver_id).unwrap().push(log_line);
    match receiver_type {
        NodeType::Client => {
            data.client_stats
                .get_mut(&receiver_id)
                .unwrap()
                .packets_received[stat_index] += 1;
        }
        NodeType::Server => {
            // TODO: serve stats
            // data.server_stats
            //     .get_mut(&receiver_id)
            //     .unwrap()
            //     .packets_received[stat_index] += 1;
        }
        NodeType::Drone => {
            unreachable!()
        }
    }
    data.ctx.request_repaint();
}

fn get_log_line_packet_received(p: &Packet, receiver_id: NodeId) -> String {
    let from_str = if is_shortcut(&p, receiver_id) {
        "SimulationController".to_string()
    } else {
        let from_id = helper::get_from_packet_received(&p);
        format!("node #{}", from_id)
    };
    match &p.pack_type {
        PacketType::FloodResponse(f) => {
            format!(
                "Received {} from {},\npath trace = {}",
                get_packet_type_str(&p.pack_type),
                from_str,
                format_path_trace(&f.path_trace),
            )
        }
        _ => {
            format!(
                "Received {} from {}",
                get_packet_type_str(&p.pack_type),
                from_str
            )
        }
    }
}

fn get_from_packet_received(p: &Packet) -> NodeId {
    let from_id = if let PacketType::FloodRequest(fr) = &p.pack_type {
        fr.path_trace.last().unwrap().0
    } else if p.routing_header.hop_index < p.routing_header.hops.len() - 1 {
        // sent by controller
        p.routing_header.hops[p.routing_header.hop_index]
    } else {
        p.routing_header.hops[p.routing_header.hop_index - 1]
    };
    from_id
}

fn is_shortcut(p: &Packet, receiver_id: NodeId) -> bool {
    let mut is_shortcut = true;
    if p.routing_header.hops.is_empty() {
        is_shortcut = false;
    } else {
        match p.routing_header.current_hop() {
            None => {
                // out of bound
                is_shortcut = false;
            }
            Some(hop_id) => {
                if hop_id == receiver_id {
                    is_shortcut = false;
                }
            }
        }
    }
    is_shortcut
}

// log strings and stats -----

fn get_packet_stat_index(t: &PacketType) -> usize {
    match t {
        PacketType::MsgFragment(_) => 0,
        PacketType::Ack(_) => 1,
        PacketType::Nack(_) => 2,
        PacketType::FloodRequest(_) => 3,
        PacketType::FloodResponse(_) => 4,
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

fn format_path_trace(pt: &Vec<(NodeId, NodeType)>) -> String {
    let mut res = "[ ".to_string();
    for (id, t) in pt {
        res.push(match t {
            NodeType::Client => 'C',
            NodeType::Drone => 'D',
            NodeType::Server => 'S',
        });
        res.push_str(&format!("{} ", id));
    }
    res.push(']');
    res
}

pub fn get_log_line_client_body(client_body: ClientBody) -> String {
    let mut res = "Type: ".to_string();
    let type_str = match client_body {
        ClientBody::ReqServerType => "Request server type".to_string(),
        ClientBody::ClientContent(ccb) => match ccb {
            ClientContentBody::ReqFilesList => "Content - Request files list".to_string(),
            ClientContentBody::ReqFile(f) => {
                format!("Content - Request file\nFile: {}", f)
            }
        },
        ClientBody::ClientCommunication(ccb) => match ccb {
            ClientCommunicationBody::ReqRegistrationToChat => {
                "Communication - Request registration to chat".to_string()
            }
            ClientCommunicationBody::MessageSend(cm) => {
                format!(
                    "Communication - Send message \nFrom: {}, To: {}\nMessage content: {}",
                    cm.from, cm.to, cm.message
                )
            }
            ClientCommunicationBody::ReqClientList => {
                "Communication - Request clients list".to_string()
            }
        },
    };
    res.push_str(&type_str);
    res
}

pub fn get_log_line_server_body(client_body: ServerBody) -> String {
    let mut res = "Type: ".to_string();
    let type_str = match client_body {
        ServerBody::RespServerType(t) => {
            format!("Response server type\nMessage content: {:?}", t)
        }
        ServerBody::ErrUnsupportedRequestType => "Error - Unsupported request type".to_string(),
        ServerBody::ServerContent(scb) => match scb {
            ServerContentBody::RespFilesList(v) => {
                format!("Content - Response files list\nMessage content: {:?}", v)
            }
            ServerContentBody::RespFile(v) => {
                format!("Content - Response files list\nMessage content: {:?}", v)
            }
            ServerContentBody::ErrFileNotFound => "Error - File not found".to_string(),
        },
        ServerBody::ServerCommunication(scb) => match scb {
            ServerCommunicationBody::RespClientList(v) => {
                format!(
                    "Communication - Response clients list\nMessage content: {:?}",
                    v
                )
            }
            ServerCommunicationBody::MessageReceive(cm) => {
                format!(
                    "Communication - Send message \nFrom: {}, To: {}\nMessage content: {}",
                    cm.from, cm.to, cm.message
                )
            }
            ServerCommunicationBody::ErrWrongClientId => "Error - Wrong client id".to_string(),
        },
    };
    res.push_str(&type_str);
    res
}
