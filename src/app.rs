use std::collections::HashMap;
use std::fs;
use std::mem::take;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crossbeam_channel::{unbounded, Sender};
use eframe::egui::{
    CentralPanel, Color32, Context, CursorIcon, Frame, Label, RichText, Sense, SidePanel,
    TopBottomPanel, Ui, Vec2,
};
use eframe::CreationContext;
use egui_graphs::{
    GraphView, LayoutRandom, LayoutStateRandom, SettingsInteraction, SettingsNavigation,
    SettingsStyle,
};
use petgraph::prelude::UnGraphMap;
use petgraph::stable_graph::StableUnGraph;
use petgraph::Undirected;

use drone_networks::network::init_network;
use wg_2024::config::Config;
use wg_2024::network::NodeId;

use crate::custom_edge::CustomEdgeShape;
use crate::custom_node::CustomNodeShape;
use crate::data::{DroneStats, SimulationData};
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
#[derive(PartialEq, Clone, Copy)]

enum Section {
    Control,
    Topology,
}

pub struct SimulationControllerUI {
    /// menu section
    section: Section,
    /// handling receiver threads
    ctx: Context,
    handles: Vec<JoinHandle<()>>,
    kill_senders: Vec<Sender<()>>,
    /// shared data
    simulation_data_ref: Option<Arc<Mutex<SimulationData>>>,
    nodes: HashMap<NodeId, NodeWindowState>,
    g: egui_graphs::Graph<
        (NodeId, NodeType),
        (),
        Undirected,
        u32,
        CustomNodeShape,
        CustomEdgeShape,
    >,
}

impl eframe::App for SimulationControllerUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_id_list();
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
        let config: Config = toml::from_str(&file_str).unwrap();
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

        // drone stats
        let mut stats = HashMap::new();
        for drone_id in sc.get_drone_ids() {
            stats.insert(drone_id, DroneStats::default());
        }

        // kill old receiving threads
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
            stats,
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

        // ui graph init ------------
        // fake sc graph
        let mut sc_graph: UnGraphMap<NodeId, ()> = UnGraphMap::new();
        sc_graph.add_node(12);
        sc_graph.add_node(23);
        sc_graph.add_node(34);
        sc_graph.add_node(45);
        sc_graph.add_node(56);
        sc_graph.add_node(67);
        sc_graph.add_edge(12, 34, ());
        sc_graph.add_edge(23, 34, ());
        sc_graph.add_edge(34, 45, ());
        sc_graph.add_edge(56, 45, ());
        sc_graph.add_edge(67, 45, ());

        let mut sg = StableUnGraph::default();

        // Insert nodes into the StableUnGraph
        let mut node_map = HashMap::new();
        for id in sc_graph.nodes() {
            let node_index = sg.add_node((id, NodeType::Drone));
            node_map.insert(id, node_index); // Map from old node to new node index
        }

        // Insert edges into the StableUnGraph
        for (source, target, _weight) in sc_graph.all_edges() {
            let source_index = node_map[&source];
            let target_index = node_map[&target];
            sg.add_edge(source_index, target_index, ());
        }

        self.g = egui_graphs::Graph::from(&sg);
    }

    fn control_section(&mut self, ctx: &Context) {
        // sidebar
        self.sidebar(ctx);
        // node windows
        CentralPanel::default().show(ctx, |_ui| {
            for id in self.get_all_ids() {
                self.spawn_node_window(ctx, id);
            }
        });
    }

    fn topology_section(&mut self, ctx: &Context) {
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
                    >::new(&mut self.g)
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
