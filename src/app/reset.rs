use crate::app::simulation_controller_ui::{
    ClientWindowState, DroneWindowState, SimulationControllerUI,
};
use crate::shared_data::{ClientStats, DroneStats, ServerStats, SimulationData};
use crate::receiver_threads;
use crossbeam_channel::unbounded;
use drone_network::controller::SimulationController;
use drone_network::network::{init_network, init_network_with_drone};
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
use lockheedrustin_drone::LockheedRustin;

impl SimulationControllerUI {
    pub fn reset_with_our_drone(&mut self) {
        self.reset(false);
    }
    pub fn reset_with_fair_drones(&mut self) {
        self.reset(true);
    }

    fn reset(&mut self, random_drones: bool) {
        self.kill_old_receiving_threads();
        // delete all file windows
        self.files.clear();

        let sc = Self::get_simulation_controller(random_drones);
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
            let arc = arc_clone;
            let event_recv = drone_receiver;
            let kill_recv = kill_drone_recv;
            receiver_threads::drone_receiver_loop(&arc, &event_recv, &kill_recv);
        });
        self.handles.push(handle);

        let arc_clone = self.simulation_data_ref.clone().unwrap();
        let handle = std::thread::spawn(move || {
            let arc = arc_clone;
            let event_recv = client_receiver;
            let kill_recv = kill_client_recv;
            receiver_threads::client_receiver_loop(&arc, &event_recv, &kill_recv);
        });
        self.handles.push(handle);

        let arc_clone = self.simulation_data_ref.clone().unwrap();
        let handle = std::thread::spawn(move || {
            let arc = arc_clone;
            let event_recv = server_receiver;
            let kill_recv = kill_server_recv;
            receiver_threads::server_receiver_loop(&arc, &event_recv, &kill_recv);
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

    fn get_simulation_controller(random_drones: bool) -> SimulationController {
        let file_str = fs::read_to_string("config.toml").unwrap();
        let config: Config = toml::from_str(&file_str).unwrap();
        if random_drones {
            init_network(&config).unwrap()
        } else {
            init_network_with_drone::<LockheedRustin>(&config, "Lockheed Rustin".to_string()).unwrap()
        }
    }

    fn reset_ids(&mut self, sc: &SimulationController) {
        self.nodes.clear();
        for id in sc.get_drone_ids() {
            self.nodes.insert(
                id,
                crate::app::simulation_controller_ui::NodeWindowState::Drone(
                    false,
                    DroneWindowState {
                        name: sc.get_group_name(id).unwrap().to_string(),
                        pdr_slider: sc.get_pdr(id).unwrap(),
                        add_link_selected_id: None,
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
