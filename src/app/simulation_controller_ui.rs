use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crossbeam_channel::Sender;
use eframe::egui::{
    CentralPanel, Color32, Context, CursorIcon, Frame, Label, RichText, Sense, SidePanel,
    TopBottomPanel, Ui, Vec2,
};
use eframe::CreationContext;
use egui_graphs::{
    GraphView, LayoutRandom, LayoutStateRandom, SettingsInteraction, SettingsNavigation,
    SettingsStyle,
};
use petgraph::stable_graph::StableUnGraph;
use petgraph::Undirected;

use petgraph::graph::NodeIndex;
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType;

use crate::data::SimulationData;
use crate::ui_components;
use crate::ui_components::client_window::{CommunicationChoice, ContentChoice, MessageChoice};
use crate::ui_components::custom_edge::CustomEdgeShape;
use crate::ui_components::custom_node::CustomNodeShape;

#[derive(Debug)]
pub(crate) enum NodeWindowState {
    Drone(bool, DroneWindowState),
    Client(bool, ClientWindowState),
    Server(bool),
}

#[derive(Default, Debug)]
pub struct ClientWindowState {
    pub message_choice: MessageChoice,
    pub content_choice: ContentChoice,
    pub communication_choice: CommunicationChoice,
    pub server_destination_id: Option<NodeId>,
    pub client_destination_id: Option<NodeId>,
    pub text_input: String,
}

#[derive(Default, Debug)]
pub struct DroneWindowState {
    pub name: String,
    pub pdr_slider: f32,
    pub add_link_selected_id: Option<NodeId>,
}

#[derive(PartialEq, Clone, Copy)]

pub(crate) enum Section {
    Control,
    Topology,
}

pub struct SimulationControllerUI {
    /// menu section
    pub(crate) section: Section,
    /// handling receiver threads
    pub(crate) ctx: Context,
    pub(crate) handles: Vec<JoinHandle<()>>,
    pub(crate) kill_senders: Vec<Sender<()>>,
    /// shared data
    pub(crate) simulation_data_ref: Option<Arc<Mutex<SimulationData>>>,
    pub(crate) nodes: HashMap<NodeId, NodeWindowState>,
    pub(crate) graph: egui_graphs::Graph<
        (NodeId, NodeType),
        (),
        Undirected,
        usize,
        CustomNodeShape,
        CustomEdgeShape,
    >,
    pub(crate) graph_index_map: HashMap<NodeId, usize>,
}

impl eframe::App for SimulationControllerUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.menu_bar(ctx);
        match self.section {
            Section::Control => {
                self.control_section(ctx);
            }
            Section::Topology => {
                self.topology_section(ctx);
            }
        }
    }
}

impl SimulationControllerUI {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let mut res = Self {
            section: Section::Control,
            ctx: cc.egui_ctx.clone(),
            handles: Vec::default(),
            kill_senders: Vec::default(),
            simulation_data_ref: None,
            nodes: HashMap::default(),
            graph: egui_graphs::Graph::from(&StableUnGraph::default()),
            graph_index_map: HashMap::default(),
        };
        res.reset_with_fair_drones();
        res
    }

    fn control_section(&mut self, ctx: &Context) {
        self.update_id_list();
        // sidebar
        self.sidebar(ctx);
        // node windows
        CentralPanel::default().show(ctx, |_ui| {
            self.spawn_node_windows(ctx);
        });
    }

    fn topology_section(&mut self, ctx: &Context) {
        self.update_graph();
        TopBottomPanel::bottom("top-panel").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.label(
                "Tip: use ctrl + mouse wheel to zoom in/out. \
                You can move nodes around and pan the camera with the mouse cursor.",
            );
            ui.add_space(2.0);
        });
        CentralPanel::default()
            .frame(Frame::default().fill(Color32::from_rgb(27, 27, 27)))
            .show(ctx, |ui| {
                ui.add(
                    &mut GraphView::<
                        (NodeId, NodeType),
                        _,
                        _,
                        _,
                        CustomNodeShape,
                        CustomEdgeShape,
                        LayoutStateRandom,
                        LayoutRandom,
                    >::new(&mut self.graph)
                    .with_styles(&SettingsStyle::default().with_labels_always(true))
                    .with_interactions(&SettingsInteraction::default().with_dragging_enabled(true))
                    .with_navigations(
                        &SettingsNavigation::default()
                            .with_fit_to_screen_enabled(false)
                            .with_zoom_and_pan_enabled(true),
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
                    self.spawn_menu_element(ui, "Control", Section::Control);
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

    pub fn sidebar(&mut self, ctx: &Context) {
        SidePanel::left("left").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.heading("Clients");
            ui.indent("clients", |ui| {
                let mut v = self.get_ids(NodeType::Client);
                v.sort_unstable();
                for id in v {
                    self.spawn_node_list_element(ui, id, "Client");
                }
            });
            ui.separator();

            ui.heading("Servers");
            ui.indent("servers", |ui| {
                let mut v = self.get_ids(NodeType::Server);
                v.sort_unstable();
                for id in v {
                    self.spawn_node_list_element(ui, id, "Server");
                }
            });
            ui.separator();

            ui.heading("Drones");
            ui.indent("drones", |ui| {
                let mut v = self.get_ids(NodeType::Drone);
                v.sort_unstable();
                for id in v {
                    self.spawn_node_list_element(ui, id, "Drone");
                }
            });
            ui.separator();
            if ui.button("Clear all logs").clicked() {
                let binding = self.simulation_data_ref.clone().unwrap();
                let mut mutex = binding.lock().unwrap();
                mutex.clear_all_logs();
            }
            ui.add_space(3.0);
            if ui.button("Reset simulation with\nfair drones").clicked() {
                self.reset_with_fair_drones();
            }
            ui.add_space(3.0);
            if ui.button("Reset simulation with\nLockheed Rustin drone").clicked() {
                self.reset_with_our_drone();
            }
            ui.add_space(3.0);
            if ui.button("Quit app").clicked() {
                std::process::exit(0);
            }
        });
    }

    pub fn spawn_node_windows(&mut self, ctx: &Context) {
        // TODO: very inefficient
        let mut sorted_node_ids = self.get_all_ids();
        sorted_node_ids.sort_unstable();

        let mut sorted_client_ids: Vec<NodeId> = self.get_ids(NodeType::Client);
        sorted_client_ids.sort_unstable();

        let mut sorted_server_ids: Vec<NodeId> = self.get_ids(NodeType::Server);
        sorted_server_ids.sort_unstable();

        let binding = self.simulation_data_ref.clone().unwrap();
        let mut mutex = binding.lock().unwrap();

        for id in self.get_all_ids() {
            match self.nodes.get_mut(&id).unwrap() {
                NodeWindowState::Drone(open, state) => {
                    ui_components::drone_window::spawn(
                        ctx,
                        &mut mutex,
                        id,
                        &sorted_node_ids,
                        open,
                        state,
                    );
                }
                NodeWindowState::Client(open, state) => {
                    ui_components::client_window::spawn(
                        ctx,
                        &mut mutex,
                        id,
                        &sorted_client_ids,
                        &sorted_server_ids,
                        open,
                        state,
                    );
                }
                NodeWindowState::Server(open) => {
                    ui_components::server_window::spawn(ctx, &mut mutex, open, id);
                }
            }
        }
    }

    fn spawn_node_list_element(&mut self, ui: &mut Ui, id: NodeId, s: &'static str) {
        ui.add_space(5.0);

        let mut drone_name = None;
        let open = match self.nodes.get_mut(&id).unwrap() {
            NodeWindowState::Client(o, _) | NodeWindowState::Server(o) => o,
            NodeWindowState::Drone(o, dws) => {
                drone_name = Some(&dws.name);
                o
            }
        };
        let marker = if *open { "> " } else { "" };

        let response = match drone_name {
            None => {
                ui.add(Label::new(format!("{marker}{s} #{id}")).sense(Sense::click()))
            }
            Some(name) => {
                ui.add(Label::new(format!("{marker}{name} #{id}")).sense(Sense::click()))
            }
        };

        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        if response.clicked() {
            *open = !*open;
        };
        ui.add_space(5.0);
    }

    pub(crate) fn get_ids(&self, node_type: NodeType) -> Vec<NodeId> {
        let mut res = vec![];
        for (id, state) in &self.nodes {
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

    fn update_graph(&mut self) {
        self.update_id_list();
        // delete crashed drones
        let current_drone_ids = self.get_ids(NodeType::Drone);
        let graph_nodes: Vec<(NodeIndex<usize>, NodeId, NodeType)> = self
            .graph
            .nodes_iter()
            .map(|(x, y)| (x, y.payload().0, y.payload().1))
            .collect();
        for (index, node_id, node_type) in graph_nodes {
            if node_type == NodeType::Drone && !current_drone_ids.contains(&node_id) {
                self.graph.remove_node(index);
                self.graph_index_map.remove(&node_id);
            }
        }
        // add new edges
        let binding = self.simulation_data_ref.clone().unwrap();
        let mutex = binding.lock().unwrap();
        let current_edges: Vec<(NodeId, NodeId, _)> = mutex.sc.get_topology().all_edges().collect();

        for (id1, id2, ()) in current_edges {
            let i1 = NodeIndex::from(*self.graph_index_map.get(&id1).unwrap());
            let i2 = NodeIndex::from(*self.graph_index_map.get(&id2).unwrap());
            let are_connected = self.graph.edges_connecting(i1, i2).count() > 0;
            if !are_connected {
                self.graph.add_edge(i1, i2, ());
            }
        }
    }
}
