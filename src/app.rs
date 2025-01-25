use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use eframe::egui::{
    CentralPanel, Context, CursorIcon, Label, Sense, SidePanel, Ui,
};
use eframe::CreationContext;

use wg_2024::network::NodeId;
use drone_networks::controller::SimulationController;

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
    simulation_data_ref: Arc<Mutex<SimulationData>>,
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
                self.drone_window(ctx, id);
            }
            for id in self.get_ids(NodeType::Server) {
                self.server_window(ctx, id);
            }
            for id in self.get_ids(NodeType::Client) {
                self.client_window(ctx, id);
            }
        });
    }
}

impl SimulationControllerUI {
    pub fn new(cc: &CreationContext<'_>, sc: SimulationController) -> Self {
        // get all node ids
        let mut types = HashMap::new();
        for id in sc.get_drone_ids() {
            types.insert(id, NodeType::Drone);
        }
        for id in sc.get_client_ids() {
            types.insert(id, NodeType::Client);
        }
        for id in sc.get_server_ids() {
            types.insert(id, NodeType::Server);
        }

        // create node hashmaps
        let mut logs = HashMap::new();
        let mut open_windows = HashMap::new();
        let mut add_link_selected_ids = HashMap::new();
        for &id in types.keys() {
            logs.insert(id, vec![]);
            open_windows.insert(id, false);
            add_link_selected_ids.insert(id, None);
        }
        // create drone hashmaps
        let mut stats = HashMap::new();
        for id in sc.get_drone_ids() {
            stats.insert(id, DroneStats::default());
        }
        let mut drone_pdr_sliders = HashMap::new();
        for drone_id in sc.get_drone_ids() {
            if let Some(pdr) = sc.get_pdr(drone_id) {
                drone_pdr_sliders.insert(drone_id, pdr);
            }
        }
        // create client hashmaps
        // let mut client_command_lines = HashMap::new();
        // for id in sc.get_drone_ids() {
        //     client_command_lines.insert(id, "".to_string());
        // }

        // create shared data and spawn threads
        let drone_receiver = sc.get_drone_recv();
        let client_receiver = sc.get_client_recv();
        let server_receiver = sc.get_server_recv();

        let data_ref = Arc::new(Mutex::new(SimulationData::new(
            sc,
            logs,
            stats,
            cc.egui_ctx.clone(),
        )));

        let tmp_clone = Arc::clone(&data_ref);
        std::thread::spawn(move || {
            receiver_threads::drone_receiver_loop(tmp_clone, drone_receiver);
        });

        let tmp_clone = Arc::clone(&data_ref);
        std::thread::spawn(move || {
            receiver_threads::client_receiver_loop(tmp_clone, client_receiver);
        });

        let tmp_clone = Arc::clone(&data_ref);
        std::thread::spawn(move || {
            receiver_threads::server_receiver_loop(tmp_clone, server_receiver);
        });

        // return
        Self {
            types,
            simulation_data_ref: data_ref,
            open_windows,
            // client_command_lines,
            drone_pdr_sliders,
            add_link_selected_ids,
        }
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

            // Quit button
            if ui.button("Quit").clicked() {
                std::process::exit(0);
            }
        });
    }

    pub fn spawn_client_window(&mut self, ctx: &Context, id: NodeId) {
        let open = self.open_windows.get_mut(&id).unwrap();
        let mutex = self.simulation_data_ref.lock().unwrap();
        ui_components::client_window::spawn_client_window(ctx, mutex, open, id);
    }

    pub fn spawn_server_window(&mut self, ctx: &Context, id: NodeId) {
        let open = self.open_windows.get_mut(&id).unwrap();
        let mutex = self.simulation_data_ref.lock().unwrap();
        ui_components::server_window::spawn_server_window(ctx, mutex, open, id);
    }

    pub fn spawn_drone_window(&mut self, ctx: &Context, id: NodeId) {
        let mut node_ids: Vec<NodeId> = self.get_all_ids();
        node_ids.sort();
        let open = self.open_windows.get_mut(&id).unwrap();
        // TODO: show only not neighbor nodes
        let selected_id = self.add_link_selected_ids.get_mut(&id).unwrap();
        let pdr_slider = self.drone_pdr_sliders.get_mut(&id).unwrap();
        let mutex = self.simulation_data_ref.lock().unwrap();
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
        self.types.iter().map(|(x, _)| *x).collect()
    }

    fn update_id_list(&mut self) {
        let mutex = self.simulation_data_ref.lock().unwrap();
        // delete crashed drones
        let sc_drone_ids = mutex.sc.get_drone_ids();
        for id in self.get_ids(NodeType::Drone) {
            if !sc_drone_ids.contains(&id) {
                self.types.remove(&id);
            }
        }
    }
}
