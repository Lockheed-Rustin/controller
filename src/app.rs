use std::collections::HashMap;
use std::fs;
use std::mem::take;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crossbeam_channel::{unbounded, Sender};
use eframe::egui::{CentralPanel, Context, CursorIcon, Label, Sense, SidePanel, Ui};
use eframe::CreationContext;

use drone_networks::network::init_network;
use wg_2024::network::NodeId;

use crate::data::{ClientStats, DroneStats, SimulationData};
use crate::receiver_threads;
use crate::ui_components;
use crate::ui_components::client_window::{CommunicationChoice, ContentChoice, MessageChoice};

#[derive(Debug)]
enum NodeWindowState {
    Drone(bool, DroneWindowState),
    Client(bool, ClientWindowState),
    Server(bool),
}

#[derive(Default, Debug)]
pub struct ClientWindowState {
    pub message_choice: MessageChoice,
    pub content_choice: ContentChoice,
    pub communication_choice: CommunicationChoice,
    pub destination_id: Option<NodeId>,
    pub text_input: String,
}

#[derive(Default, Debug)]
pub struct DroneWindowState {
    pub pdr_slider: f32,
    pub add_link_selected_id: Option<NodeId>,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum NodeType {
    Client,
    Drone,
    Server,
}

pub struct SimulationControllerUI {
    ctx: Context,
    /// handling receiver threads
    handles: Vec<JoinHandle<()>>,
    kill_senders: Vec<Sender<()>>,
    /// shared data
    simulation_data_ref: Option<Arc<Mutex<SimulationData>>>,
    nodes: HashMap<NodeId, NodeWindowState>,
}

impl eframe::App for SimulationControllerUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_id_list();
        // sidebar
        self.sidebar(ctx);
        // node windows
        CentralPanel::default().show(ctx, |_ui| {
            for id in self.get_all_ids() {
                self.spawn_node_window(ctx, id);
            }
        });
    }
}

impl SimulationControllerUI {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let mut res = Self {
            ctx: cc.egui_ctx.clone(),
            handles: Default::default(),
            kill_senders: Default::default(),
            simulation_data_ref: None,
            nodes: Default::default(),
        };
        res.reset();
        res
    }

    pub fn reset(&mut self) {
        // read config file and get a SimulationController
        let file_str = fs::read_to_string("config.toml").unwrap();
        let config = toml::from_str(&file_str).unwrap();
        let sc = init_network(&config).unwrap();

        // get all node ids
        self.nodes.clear();
        for id in sc.get_drone_ids() {
            self.nodes.insert(
                id,
                NodeWindowState::Drone(false, DroneWindowState::default()),
            );
        }
        for id in sc.get_client_ids() {
            self.nodes.insert(
                id,
                NodeWindowState::Client(false, ClientWindowState::default()),
            );
        }
        for id in sc.get_server_ids() {
            self.nodes.insert(id, NodeWindowState::Server(false));
        }

        // node logs
        let mut logs = HashMap::new();
        for &id in self.nodes.keys() {
            logs.insert(id, vec![]);
        }

        // stats
        let mut drone_stats = HashMap::new();
        for drone_id in sc.get_drone_ids() {
            drone_stats.insert(drone_id, DroneStats::default());
        }
        let mut client_stats = HashMap::new();
        for client_id in sc.get_client_ids() {
            client_stats.insert(client_id, ClientStats::default());
        }

        // kill receiving threads
        for s in self.kill_senders.iter() {
            s.send(())
                .expect("Error in sending kill message to receiving threads");
        }
        let handles = take(&mut self.handles);
        for h in handles {
            h.join().expect("Error in joining receiving threads");
        }
        self.handles.clear();
        self.kill_senders.clear();

        // create shared data and spawn threads
        let drone_receiver = sc.get_drone_recv();
        let client_receiver = sc.get_client_recv();
        let server_receiver = sc.get_server_recv();

        let (kill_client_send, kill_client_recv) = unbounded();
        let (kill_server_send, kill_server_recv) = unbounded();
        let (kill_drone_send, kill_drone_recv) = unbounded();
        self.kill_senders.push(kill_client_send);
        self.kill_senders.push(kill_server_send);
        self.kill_senders.push(kill_drone_send);

        self.simulation_data_ref = Some(Arc::new(Mutex::new(SimulationData::new(
            sc,
            logs,
            drone_stats,
            client_stats,
            self.ctx.clone(),
        ))));

        let tmp_clone = self.simulation_data_ref.clone().unwrap();
        let handle = std::thread::spawn(move || {
            receiver_threads::drone_receiver_loop(tmp_clone, drone_receiver, kill_drone_recv);
        });
        self.handles.push(handle);

        let tmp_clone = self.simulation_data_ref.clone().unwrap();
        let handle = std::thread::spawn(move || {
            receiver_threads::client_receiver_loop(tmp_clone, client_receiver, kill_client_recv);
        });
        self.handles.push(handle);

        let tmp_clone = self.simulation_data_ref.clone().unwrap();
        let handle = std::thread::spawn(move || {
            receiver_threads::server_receiver_loop(tmp_clone, server_receiver, kill_server_recv);
        });
        self.handles.push(handle);
    }

    pub fn sidebar(&mut self, ctx: &Context) {
        SidePanel::left("left").show(ctx, |ui| {
            ui.heading("Nodes");
            ui.separator();

            ui.label("Clients");
            ui.indent("clients", |ui| {
                for id in self.get_ids(NodeType::Client) {
                    self.spawn_node_list_element(ui, id, "Client");
                }
            });
            ui.separator();

            ui.label("Servers");
            ui.indent("servers", |ui| {
                for id in self.get_ids(NodeType::Server) {
                    self.spawn_node_list_element(ui, id, "Server");
                }
            });
            ui.separator();

            ui.label("Drones");
            ui.indent("drones", |ui| {
                // clickable labels
                for id in self.get_ids(NodeType::Drone) {
                    self.spawn_node_list_element(ui, id, "Drone");
                }
            });
            ui.separator();

            if ui.button("Reset").clicked() {
                self.reset();
            }
            if ui.button("Quit").clicked() {
                std::process::exit(0);
            }
        });
    }

    pub fn spawn_node_window(&mut self, ctx: &Context, id: NodeId) {
        let mut node_ids: Vec<NodeId> = self.get_all_ids();
        node_ids.sort();
        let binding = self.simulation_data_ref.clone().unwrap();
        let mut mutex = binding.lock().unwrap();
        match self.nodes.get_mut(&id).unwrap() {
            NodeWindowState::Drone(open, state) => {
                ui_components::drone_window::spawn_drone_window(
                    ctx, &mut mutex, id, node_ids, open, state,
                );
            }
            NodeWindowState::Client(open, state) => {
                ui_components::client_window::spawn_client_window(
                    ctx, &mut mutex, id, node_ids, open, state,
                );
            }
            NodeWindowState::Server(open) => {
                ui_components::server_window::spawn_server_window(ctx, &mut mutex, open, id);
            }
        }
    }

    fn spawn_node_list_element(&mut self, ui: &mut Ui, id: NodeId, s: &'static str) {
        ui.add_space(5.0);
        let response = ui.add(Label::new(format!("{} #{}", s, id)).sense(Sense::click()));
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        if response.clicked() {
            match self.nodes.get_mut(&id).unwrap() {
                NodeWindowState::Drone(o, _) => *o = true,
                NodeWindowState::Client(o, _) => *o = true,
                NodeWindowState::Server(o) => *o = true,
            }
        };
        ui.add_space(5.0);
    }

    fn get_ids(&self, node_type: NodeType) -> Vec<NodeId> {
        let mut res = vec![];
        for (id, state) in self.nodes.iter() {
            match state {
                NodeWindowState::Drone(_, _) => {
                    if node_type == NodeType::Drone {
                        res.push(*id);
                    }
                }
                NodeWindowState::Client(_, _) => {
                    if node_type == NodeType::Client {
                        res.push(*id);
                    }
                }
                NodeWindowState::Server(_) => {
                    if node_type == NodeType::Server {
                        res.push(*id);
                    }
                }
            }
        }
        res
    }

    fn get_all_ids(&self) -> Vec<NodeId> {
        self.nodes.keys().copied().collect()
    }

    fn update_id_list(&mut self) {
        let binding = self.simulation_data_ref.clone().unwrap();
        let mutex = binding.lock().unwrap();
        // delete crashed drones
        let sc_drone_ids = mutex.sc.get_drone_ids();
        for id in self.get_ids(NodeType::Drone) {
            if !sc_drone_ids.contains(&id) {
                self.nodes.remove(&id);
            }
        }
    }
}
