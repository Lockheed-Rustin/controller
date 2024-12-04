use std::collections::HashMap;

use eframe::egui::Context;

use wg_2024::network::NodeId;
use wg_2024::packet::{Packet, PacketType};

pub struct SimulationData {
    pub logs: HashMap<NodeId, Vec<String>>,
    pub stats: HashMap<NodeId, DroneStats>,
    pub ctx: Context,
}

impl SimulationData {
    pub fn new(
        logs: HashMap<NodeId, Vec<String>>,
        stats: HashMap<NodeId, DroneStats>,
        ctx: Context,
    ) -> Self {
        Self { logs, stats, ctx }
    }
}

#[derive(Default)]
pub struct DroneStats {
    packets_forwarded: HashMap<PacketType, u32>,
    packets_dropped: HashMap<PacketType, u32>,
}

#[derive(Default)]
pub struct ClientStats {
    packets_sent: HashMap<PacketType, u32>,
    packets_received: HashMap<PacketType, u32>,
}

#[derive(Default)]
pub struct ServerStats {
    packets_sent: HashMap<PacketType, u32>,
    packets_received: HashMap<PacketType, u32>,
}