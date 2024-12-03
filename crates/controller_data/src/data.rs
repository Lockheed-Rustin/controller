use std::collections::HashMap;

use eframe::egui::Context;

use drone_networks::controller::SimulationController;
use wg_2024::network::NodeId;


pub struct SimulationData {
    pub logs : HashMap<NodeId, String>,
    pub stats: HashMap<NodeId, DroneStats>,
    pub ctx: Context,
}

impl SimulationData {
    pub fn new(logs: HashMap<NodeId, String>, stats: HashMap<NodeId, DroneStats>, ctx: Context) -> Self {
        Self {
            logs,
            stats,
            ctx
        }
    }
}

#[derive(Default)]
pub struct DroneStats {
    forwarded_packets: u32,
    dropped_packets: u32,
}
