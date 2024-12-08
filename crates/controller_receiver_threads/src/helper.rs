use wg_2024::network::NodeId;
use wg_2024::packet::{Packet, PacketType};

// all nodes
pub fn get_log_line_packet_sent(p: &Packet, to_id: Option<NodeId>) -> String {
    match to_id {
        None => format!("{} broadcasted", get_packet_type_str(&p.pack_type)),
        Some(id) => format!("{} sent to node #{}", get_packet_type_str(&p.pack_type), id),
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
    format!(
        "Received {} from node #{}",
        get_packet_type_str(&p.pack_type),
        from_id
    )
}

pub fn get_from_packet_received(p: &Packet) -> NodeId {
    let from_id = if let PacketType::FloodRequest(fr) = &p.pack_type {
        fr.path_trace.last().unwrap().0
    } else {
        if p.routing_header.hop_index < p.routing_header.hops.len() - 1 { // sent by controller
            p.routing_header.hops[p.routing_header.hop_index]
        } else {
            p.routing_header.hops[p.routing_header.hop_index - 1]
        }
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
