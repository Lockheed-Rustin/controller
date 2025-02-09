use crate::app::simulation_controller_ui::{
    ClientWindowState, DroneWindowState, SimulationControllerUI,
};
use crate::data::{ClientStats, DroneStats, ServerStats, SimulationData};
use crate::receiver_threads;
use crossbeam_channel::unbounded;
use drone_networks::controller::SimulationController;
use drone_networks::network::init_network;
use eframe::egui::Color32;
use petgraph::graph::NodeIndex;
use petgraph::graphmap::UnGraphMap;
use petgraph::prelude::StableUnGraph;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::mem::take;
use std::sync::{Arc, Mutex};
use wg_2024::config::Config;
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType;

impl SimulationControllerUI {
    pub fn reset(&mut self) {
        self.kill_old_receiving_threads();

        let sc = Self::get_simulation_controller();
        self.reset_ids(&sc);

        // new shared data
        let logs = self.get_new_logs();
        let drone_stats = self.get_new_drone_stats();
        let client_stats = self.get_new_client_stats();
        let server_stats = self.get_new_server_stats();

        self.reset_graph(&sc);

        // create channels
        let drone_receiver = sc.get_drone_recv();
        let client_receiver = sc.get_client_recv();
        let server_receiver = sc.get_server_recv();

        let (kill_client_send, kill_client_recv) = unbounded();
        let (kill_server_send, kill_server_recv) = unbounded();
        let (kill_drone_send, kill_drone_recv) = unbounded();
        self.kill_senders.push(kill_client_send);
        self.kill_senders.push(kill_server_send);
        self.kill_senders.push(kill_drone_send);

        // create shared data
        self.simulation_data_ref = Some(Arc::new(Mutex::new(SimulationData::new(
            sc,
            logs,
            drone_stats,
            client_stats,
            server_stats,
            self.ctx.clone(),
        ))));

        // spawn receiving threads
        let arc_clone = self.simulation_data_ref.clone().unwrap();
        let handle = std::thread::spawn(move || {
            receiver_threads::drone_receiver_loop(arc_clone, drone_receiver, kill_drone_recv);
        });
        self.handles.push(handle);

        let arc_clone = self.simulation_data_ref.clone().unwrap();
        let handle = std::thread::spawn(move || {
            receiver_threads::client_receiver_loop(arc_clone, client_receiver, kill_client_recv);
        });
        self.handles.push(handle);

        let arc_clone = self.simulation_data_ref.clone().unwrap();
        let handle = std::thread::spawn(move || {
            receiver_threads::server_receiver_loop(arc_clone, server_receiver, kill_server_recv);
        });
        self.handles.push(handle);
    }

    fn kill_old_receiving_threads(&mut self) {
        // kill old receiving threads
        for s in &self.kill_senders {
            s.send(())
                .expect("Error in sending kill message to receiving threads");
        }
        let handles = take(&mut self.handles);
        for h in handles {
            h.join().expect("Error in joining receiving threads");
        }
        self.handles.clear();
        self.kill_senders.clear();
    }

    fn get_simulation_controller() -> SimulationController {
        let file_str = fs::read_to_string("config.toml").unwrap();
        let config: Config = toml::from_str(&file_str).unwrap();
        init_network(&config).unwrap()
    }

    fn reset_ids(&mut self, sc: &SimulationController) {
        self.nodes.clear();
        for id in sc.get_drone_ids() {
            self.nodes.insert(
                id,
                crate::app::simulation_controller_ui::NodeWindowState::Drone(
                    false,
                    DroneWindowState {
                        pdr_slider: sc.get_pdr(id).unwrap(),
                        ..Default::default()
                    },
                ),
            );
        }
        for id in sc.get_client_ids() {
            self.nodes.insert(
                id,
                crate::app::simulation_controller_ui::NodeWindowState::Client(
                    false,
                    ClientWindowState::default(),
                ),
            );
        }
        for id in sc.get_server_ids() {
            self.nodes.insert(
                id,
                crate::app::simulation_controller_ui::NodeWindowState::Server(false),
            );
        }
    }

    fn get_new_logs(&self) -> HashMap<NodeId, VecDeque<(String, Color32)>> {
        let mut logs = HashMap::new();
        for &id in self.nodes.keys() {
            logs.insert(id, VecDeque::new());
        }
        logs
    }

    fn get_new_drone_stats(&self) -> HashMap<NodeId, DroneStats> {
        let mut drone_stats = HashMap::new();
        for drone_id in self.get_ids(NodeType::Drone) {
            drone_stats.insert(drone_id, DroneStats::default());
        }
        drone_stats
    }
    fn get_new_client_stats(&self) -> HashMap<NodeId, ClientStats> {
        let mut client_stats = HashMap::new();
        for client_id in self.get_ids(NodeType::Client) {
            client_stats.insert(client_id, ClientStats::default());
        }
        client_stats
    }
    fn get_new_server_stats(&self) -> HashMap<NodeId, ServerStats> {
        let mut server_stats = HashMap::new();
        for server_id in self.get_ids(NodeType::Server) {
            server_stats.insert(server_id, ServerStats::default());
        }
        server_stats
    }

    fn reset_graph(&mut self, sc: &SimulationController) {
        let sc_graph: &UnGraphMap<NodeId, ()> = sc.get_topology();

        let mut sg = StableUnGraph::default();

        // Insert nodes into the StableUnGraph
        self.graph_index_map.clear();
        for id in self.get_ids(NodeType::Drone) {
            let node_index = sg.add_node((id, NodeType::Drone));
            self.graph_index_map.insert(id, node_index.index());
        }
        for id in self.get_ids(NodeType::Client) {
            let node_index = sg.add_node((id, NodeType::Client));
            self.graph_index_map.insert(id, node_index.index());
        }
        for id in self.get_ids(NodeType::Server) {
            let node_index = sg.add_node((id, NodeType::Server));
            self.graph_index_map.insert(id, node_index.index());
        }

        // Insert edges into the StableUnGraph
        for (source, target, _weight) in sc_graph.all_edges() {
            let source_index = self.graph_index_map[&source];
            let target_index = self.graph_index_map[&target];
            sg.add_edge(
                NodeIndex::from(source_index),
                NodeIndex::from(target_index),
                (),
            );
        }

        self.graph = egui_graphs::Graph::from(&sg);
    }
}
