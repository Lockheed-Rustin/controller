use std::collections::HashMap;

use eframe::egui::{Color32, Context};

use drone_networks::controller::SimulationController;
use wg_2024::network::NodeId;

pub struct SimulationData {
    pub sc: SimulationController,
    pub logs: HashMap<NodeId, Vec<(String, Color32)>>,
    pub drone_stats: HashMap<NodeId, DroneStats>,
    pub client_stats: HashMap<NodeId, ClientStats>,
    pub server_stats: HashMap<NodeId, ServerStats>,
    pub ctx: Context,
}

impl SimulationData {
    pub fn new(
        sc: SimulationController,
        logs: HashMap<NodeId, Vec<(String, Color32)>>,
        drone_stats: HashMap<NodeId, DroneStats>,
        client_stats: HashMap<NodeId, ClientStats>,
        server_stats: HashMap<NodeId, ServerStats>,
        ctx: Context,
    ) -> Self {
        Self {
            sc,
            logs,
            drone_stats,
            client_stats,
            server_stats,
            ctx,
        }
    }
}

#[derive(Default)]
pub struct DroneStats {
    // 0:Fragment, 1:Ack, 2:Nack, 3:Flood Req, 4:Flood Resp
    pub packets_forwarded: [u32; 5],
    pub fragments_dropped: u32,
}

#[derive(Default)]
pub struct ClientStats {
    // 0:Fragment, 1:Ack, 2:Nack, 3:Flood Req, 4:Flood Resp
    pub packets_sent: [u32; 5],
    pub packets_received: [u32; 5],
    pub messages_assembled: u32,
    pub messages_fragmented: u32,
}

#[derive(Default)]
pub struct ServerStats {
    // 0:Fragment, 1:Ack, 2:Nack, 3:Flood Req, 4:Flood Resp
    pub packets_sent: [u32; 5],
    pub packets_received: [u32; 5],
    pub messages_assembled: u32,
    pub messages_fragmented: u32,
}
