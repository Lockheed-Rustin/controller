use std::collections::{HashMap, VecDeque};

use eframe::egui::{Color32, Context};

use crate::app::simulation_controller_ui::ContentFile;
use drone_networks::controller::SimulationController;
use wg_2024::network::NodeId;

const MAX_LOG_LENGTH: usize = 100;

pub struct SimulationData {
    pub sc: SimulationController,
    logs: HashMap<NodeId, VecDeque<(String, Color32)>>,
    pub drone_stats: HashMap<NodeId, DroneStats>,
    pub client_stats: HashMap<NodeId, ClientStats>,
    pub server_stats: HashMap<NodeId, ServerStats>,
    pub ctx: Context,
    pub files: Vec<ContentFile>,
}

impl SimulationData {
    #[must_use]
    pub fn new(
        sc: SimulationController,
        logs: HashMap<NodeId, VecDeque<(String, Color32)>>,
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
            files: vec![]
        }
    }

    /// adds a log entry for the node that matches id.
    /// # Panics
    /// Will panic if the id is not present.
    pub fn add_log(&mut self, id: NodeId, str: String, color: Color32) {
        let v = self.logs.get_mut(&id).unwrap();
        if v.len() >= MAX_LOG_LENGTH {
            v.pop_front();
        }
        v.push_back((str, color));
    }

    /// clears logs for the node that matches id.
    /// # Panics
    /// Will panic if the id is not present.
    pub fn clear_log(&mut self, id: NodeId) {
        let v = self.logs.get_mut(&id).unwrap();
        v.clear();
    }

    pub fn clear_all_logs(&mut self) {
        for v in self.logs.values_mut() {
            v.clear();
        }
    }

    /// Returns immutable borrow of the `VecDeque` containing all logs
    /// for the node that matches id.
    /// # Panics
    /// Will panic if the id is not present.
    #[must_use]
    pub fn get_logs(&self, id: NodeId) -> &VecDeque<(String, Color32)> {
        self.logs.get(&id).unwrap()
    }
}

#[derive(Default)]
pub struct DroneStats {
    // 0:Fragment, 1:Ack, 2:Nack, 3:Flood Req, 4:Flood Resp
    pub packets_forwarded: [u64; 5],
    pub fragments_dropped: u64,
}

#[derive(Default)]
pub struct ClientStats {
    // 0:Fragment, 1:Ack, 2:Nack, 3:Flood Req, 4:Flood Resp
    pub packets_sent: [u64; 5],
    pub packets_received: [u64; 5],
    pub messages_assembled: u64,
    pub messages_fragmented: u64,
}

#[derive(Default)]
pub struct ServerStats {
    // 0:Fragment, 1:Ack, 2:Nack, 3:Flood Req, 4:Flood Resp
    pub packets_sent: [u64; 5],
    pub packets_received: [u64; 5],
    pub messages_assembled: u64,
    pub messages_fragmented: u64,
}
