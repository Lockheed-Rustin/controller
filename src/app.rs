use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eframe::egui::{
    CentralPanel, Color32, Context, CursorIcon, Frame, Label, RichText, Sense, SidePanel,
    TopBottomPanel, Ui, Vec2,
};
use eframe::CreationContext;

use drone_networks::controller::SimulationController;
use egui_graphs::{
    GraphView, LayoutRandom, LayoutStateRandom, SettingsInteraction, SettingsNavigation,
    SettingsStyle,
};
use petgraph::prelude::UnGraphMap;
use petgraph::stable_graph::StableUnGraph;
use petgraph::Undirected;
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
#[derive(PartialEq, Clone, Copy)]

enum Section {
    Nodes,
    Topology,
}

pub struct SimulationControllerUI {
    section: Section,
    simulation_data_ref: Arc<Mutex<SimulationData>>,
    // nodes
    types: HashMap<NodeId, NodeType>,
    open_windows: HashMap<NodeId, bool>,
    // clients
    // client_command_lines: HashMap<NodeId, String>,
    // drones
    drone_pdr_sliders: HashMap<NodeId, f32>,
    add_link_selected_ids: HashMap<NodeId, Option<NodeId>>,
    g: egui_graphs::Graph<NodeId, (), Undirected>,
}

impl eframe::App for SimulationControllerUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_id_list();
        self.menu_bar(ctx);
        match self.section {
            Section::Nodes => {
                self.nodes_section(ctx);
            }
            Section::Topology => {
                self.topology_section(ctx);
            }
        }
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

        // ui graph init ------------
        // fake sc graph
        let mut sc_graph: UnGraphMap<NodeId, ()> = UnGraphMap::new();
        sc_graph.add_node(12);
        sc_graph.add_node(34);
        sc_graph.add_node(56);
        sc_graph.add_edge(12, 34, ());
        sc_graph.add_edge(56, 34, ());

        let mut sg = StableUnGraph::default();

        // Insert nodes into the StableUnGraph
        let mut node_map = HashMap::new();
        for node in sc_graph.nodes() {
            let node_index = sg.add_node(node.clone());
            node_map.insert(node, node_index); // Map from old node to new node index
        }

        // Insert edges into the StableUnGraph
        for (source, target, _weight) in sc_graph.all_edges() {
            let source_index = node_map[&source];
            let target_index = node_map[&target];
            sg.add_edge(source_index, target_index, ());
        }

        // delete shitty labels
        let mut g = egui_graphs::Graph::from(&sg);
        let mut v = vec![];
        for (i, _) in g.edges_iter() {
            v.push(i);
        }
        for i in v {
            g.edge_mut(i).unwrap().set_label("".to_string());
        }

        // return
        Self {
            section: Section::Nodes,
            types,
            simulation_data_ref: data_ref,
            open_windows,
            // client_command_lines,
            drone_pdr_sliders,
            add_link_selected_ids,
            g,
        }
    }

    fn nodes_section(&mut self, ctx: &Context) {
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

    fn topology_section(&mut self, ctx: &Context) {
        CentralPanel::default()
            .frame(
                Frame::default()
                    .fill(Color32::from_rgb(27, 27, 27))
                    .inner_margin(Vec2::new(8.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.add(
                    &mut GraphView::<_, _, _, _, _, _, LayoutStateRandom, LayoutRandom>::new(
                        &mut self.g,
                    )
                    .with_styles(&SettingsStyle::default().with_labels_always(true))
                    .with_interactions(&SettingsInteraction::default().with_dragging_enabled(true))
                    .with_navigations(
                        &SettingsNavigation::default().with_fit_to_screen_enabled(false),
                    ),
                );
            });
    }

    fn menu_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("menu")
            .frame(
                Frame::default()
                    .fill(Color32::from_rgb(50, 50, 50))
                    .inner_margin(Vec2::new(8.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    self.spawn_menu_element(ui, "Nodes", Section::Nodes);
                    self.spawn_menu_element(ui, "Topology", Section::Topology);
                });
            });
    }

    fn spawn_menu_element(&mut self, ui: &mut Ui, str: &'static str, section: Section) {
        let text = if self.section == section {
            RichText::new(str).strong().underline().size(20.0)
        } else {
            RichText::new(str).size(20.0)
        };
        let response = ui.add(Label::new(text).sense(Sense::click()));
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        if response.clicked() {
            self.section = section;
        };
        ui.add_space(10.0);
    }

    fn sidebar(&mut self, ctx: &Context) {
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

    fn spawn_client_window(&mut self, ctx: &Context, id: NodeId) {
        let open = self.open_windows.get_mut(&id).unwrap();
        let mutex = self.simulation_data_ref.lock().unwrap();
        ui_components::client_window::spawn_client_window(ctx, mutex, open, id);
    }

    fn spawn_server_window(&mut self, ctx: &Context, id: NodeId) {
        let open = self.open_windows.get_mut(&id).unwrap();
        let mutex = self.simulation_data_ref.lock().unwrap();
        ui_components::server_window::spawn_server_window(ctx, mutex, open, id);
    }

    fn spawn_drone_window(&mut self, ctx: &Context, id: NodeId) {
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
        self.types.keys().copied().collect()
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
