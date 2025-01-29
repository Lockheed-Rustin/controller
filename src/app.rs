use std::collections::HashMap;
use std::fs;
use std::mem::take;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use crossbeam_channel::{unbounded, Sender};
use eframe::egui::{CentralPanel, Context, CursorIcon, Label, Sense, SidePanel, Ui};
use eframe::CreationContext;

use drone_networks::controller::SimulationController;
use drone_networks::network::init_network;
use wg_2024::network::NodeId;

use crate::data::{DroneStats, SimulationData};
use crate::receiver_threads;
use crate::ui_components;

#[derive(PartialEq, Clone, Copy)]
enum NodeType {
    Client,
    Drone,
    Server,
}

pub struct SimulationControllerUI {
    ctx: Context,
    handles: Vec<JoinHandle<()>>,
    kill_senders: Vec<Sender<()>>,
    simulation_data_ref: Option<Arc<Mutex<SimulationData>>>,
    // nodes
    types: HashMap<NodeId, NodeType>,
    open_windows: HashMap<NodeId, bool>,
    // clients
    // client_command_lines: HashMap<NodeId, String>,
    // drones
    drone_pdr_sliders: HashMap<NodeId, f32>,
    add_link_selected_ids: HashMap<NodeId, Option<NodeId>>,
}

impl eframe::App for SimulationControllerUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_id_list();
        // sidebar
        self.sidebar(ctx);
        // node windows
        CentralPanel::default().show(ctx, |_ui| {
            for id in self.get_ids(NodeType::Drone) {
                self.spawn_drone_window(ctx, id);
            }
            for id in self.get_ids(NodeType::Server) {
                self.spawn_server_window(ctx, id);
            }
            for id in self.get_ids(NodeType::Client) {
                self.spawn_client_window(ctx, id);
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
            types: Default::default(),
            open_windows: Default::default(),
            drone_pdr_sliders: Default::default(),
            add_link_selected_ids: Default::default(),
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
        self.types.clear();
        for id in sc.get_drone_ids() {
            self.types.insert(id, NodeType::Drone);
        }
        for id in sc.get_client_ids() {
            self.types.insert(id, NodeType::Client);
        }
        for id in sc.get_server_ids() {
            self.types.insert(id, NodeType::Server);
        }

        // create node hashmaps
        self.open_windows.clear();
        self.add_link_selected_ids.clear();
        let mut logs = HashMap::new();
        for &id in self.types.keys() {
            self.open_windows.insert(id, false);
            self.add_link_selected_ids.insert(id, None);
            logs.insert(id, vec![]);
        }

        // create drone hashmaps
        let mut stats = HashMap::new();
        for id in sc.get_drone_ids() {
            stats.insert(id, DroneStats::default());
        }
        self.drone_pdr_sliders.clear();
        for drone_id in sc.get_drone_ids() {
            if let Some(pdr) = sc.get_pdr(drone_id) {
                self.drone_pdr_sliders.insert(drone_id, pdr);
            }
        }
        // create client hashmaps
        // let mut client_command_lines = HashMap::new();
        // for id in sc.get_drone_ids() {
        //     client_command_lines.insert(id, "".to_string());
        // }

        // kill threads
        for s in self.kill_senders.iter() {
            s.send(()).expect("ERROR IN KILLING");
        }
        let handles = take(&mut self.handles);
        for h in handles {
            h.join().expect("ERROR IN JOIN");
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
            stats,
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
            // Quit button
            if ui.button("Quit").clicked() {
                std::process::exit(0);
            }
        });
    }

    pub fn spawn_client_window(&mut self, ctx: &Context, id: NodeId) {
        let open = self.open_windows.get_mut(&id).unwrap();
        let binding = self.simulation_data_ref.clone().unwrap();
        let mutex = binding.lock().unwrap();
        ui_components::client_window::spawn_client_window(ctx, mutex, open, id);
    }

    pub fn spawn_server_window(&mut self, ctx: &Context, id: NodeId) {
        let open = self.open_windows.get_mut(&id).unwrap();
        let binding = self.simulation_data_ref.clone().unwrap();
        let mutex = binding.lock().unwrap();
        ui_components::server_window::spawn_server_window(ctx, mutex, open, id);
    }

    pub fn spawn_drone_window(&mut self, ctx: &Context, id: NodeId) {
        let mut node_ids: Vec<NodeId> = self.get_all_ids();
        node_ids.sort();
        let open = self.open_windows.get_mut(&id).unwrap();
        // TODO: show only not neighbor nodes
        let selected_id = self.add_link_selected_ids.get_mut(&id).unwrap();
        let pdr_slider = self.drone_pdr_sliders.get_mut(&id).unwrap();
        let binding = self.simulation_data_ref.clone().unwrap();
        let mutex = binding.lock().unwrap();
        ui_components::drone_window::spawn_drone_window(
            ctx,
            mutex,
            open,
            id,
            node_ids,
            selected_id,
            pdr_slider,
        );
    }

    fn spawn_node_list_element(&mut self, ui: &mut Ui, id: NodeId, s: &'static str) {
        ui.add_space(5.0);
        let response = ui.add(Label::new(format!("{} #{}", s, id)).sense(Sense::click()));
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        if response.clicked() {
            self.open_windows.insert(id, true);
        };
        ui.add_space(5.0);
    }

    fn get_ids(&self, node_type: NodeType) -> Vec<NodeId> {
        self.types
            .iter()
            .filter(|(_, &t)| t == node_type)
            .map(|(x, _)| *x)
            .collect()
    }

    fn get_all_ids(&self) -> Vec<NodeId> {
        self.types.keys().copied().collect()
    }

    fn update_id_list(&mut self) {
        let binding = self.simulation_data_ref.clone().unwrap();
        let mutex = binding.lock().unwrap();
        // delete crashed drones
        let sc_drone_ids = mutex.sc.get_drone_ids();
        for id in self.get_ids(NodeType::Drone) {
            if !sc_drone_ids.contains(&id) {
                self.types.remove(&id);
            }
        }
    }
}
