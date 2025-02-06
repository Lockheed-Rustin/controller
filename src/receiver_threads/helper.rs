use drone_networks::message::{
    ClientBody, ClientCommunicationBody, ClientContentBody, ServerBody, ServerCommunicationBody,
    ServerContentBody,
};
use wg_2024::network::NodeId;
use wg_2024::packet::{NodeType, Packet, PacketType};

// all nodes
pub fn get_log_line_packet_sent(p: &Packet, to_id: Option<NodeId>) -> String {
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

pub fn get_from_and_to_packet_send(p: &Packet) -> (NodeId, Option<NodeId>) {
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

// clients and servers
pub fn get_log_line_packet_received(p: &Packet, from_id: NodeId) -> String {
    match &p.pack_type {
        PacketType::FloodResponse(f) => {
            format!(
                "Received {} from node #{},\npath trace = {}",
                get_packet_type_str(&p.pack_type),
                from_id,
                format_path_trace(&f.path_trace),
            )
        }
        _ => {
            format!(
                "Received {} from node #{}",
                get_packet_type_str(&p.pack_type),
                from_id
            )
        }
    }
}

pub fn get_log_line_packet_received_shortcut(p: &Packet) -> String {
    match &p.pack_type {
        PacketType::FloodResponse(f) => {
            format!(
                "Received {} from SimulationController,\npath trace = {}",
                get_packet_type_str(&p.pack_type),
                format_path_trace(&f.path_trace),
            )
        }
        _ => {
            format!(
                "Received {} from SimulationController",
                get_packet_type_str(&p.pack_type),
            )
        }
    }
}

pub fn get_from_packet_received(p: &Packet) -> NodeId {
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

pub fn get_packet_stat_index(t: &PacketType) -> usize {
    match t {
        PacketType::MsgFragment(_) => 0,
        PacketType::Ack(_) => 1,
        PacketType::Nack(_) => 2,
        PacketType::FloodRequest(_) => 3,
        PacketType::FloodResponse(_) => 4,
    }
}

pub fn get_packet_type_str(t: &PacketType) -> &'static str {
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
