use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use eframe::egui::{vec2, CentralPanel, Color32, ComboBox, Context, CursorIcon, Direction, Grid, Key, Label, Layout, RichText, ScrollArea, Sense, SidePanel, Slider, TextEdit, TextStyle, Ui, Window};
use eframe::CreationContext;

use drone_networks::controller::SimulationController;
use wg_2024::network::NodeId;

use controller_data::{DroneStats, SimulationData};
use controller_receiver_threads::{drone_receiver_loop, client_receiver_loop, server_receiver_loop};

pub struct SimulationControllerUI {
    sc: SimulationController,
    simulation_data_ref: Arc<Mutex<SimulationData>>,

    open_windows: HashMap<NodeId, bool>,
    client_command_lines: HashMap<NodeId, String>,
    drone_pdrs: HashMap<NodeId, f32>,
    add_link_selected_ids: HashMap<NodeId, Option<NodeId>>,
}

impl eframe::App for SimulationControllerUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // sidebar
        self.sidebar(ctx);
        // node windows
        CentralPanel::default().show(ctx, |_ui| {
            for id in self.sc.get_drone_ids() {
                self.drone_window(ctx, id);
            }
            for id in self.sc.get_server_ids() {
                self.server_window(ctx, id);
            }
            for id in self.sc.get_client_ids() {
                self.client_window(ctx, id);
            }
        });
    }
}

impl SimulationControllerUI {
    pub fn new(cc: &CreationContext<'_>, sc: SimulationController) -> Self {
        // get all node ids
        let mut ids = HashSet::new();
        for client_id in sc.get_client_ids() {
            ids.insert(client_id);
        }
        for drone_id in sc.get_drone_ids() {
            ids.insert(drone_id);
        }
        for server_id in sc.get_server_ids() {
            ids.insert(server_id);
        }

        // create node hashmaps
        let mut logs = HashMap::new();
        let mut open_windows = HashMap::new();
        let mut add_link_selected_ids = HashMap::new();
        for id in ids.clone() {
            logs.insert(id, vec![]);
            open_windows.insert(id, false);
            add_link_selected_ids.insert(id, None);
        }
        // create drone hashmaps
        let mut stats = HashMap::new();
        let mut drone_pdrs = HashMap::new();
        for id in sc.get_drone_ids() {
            stats.insert(id, DroneStats::default());
            drone_pdrs.insert(id, 0.0);
        }
        // create client hashmaps
        let mut client_command_lines = HashMap::new();
        for id in sc.get_drone_ids() {
            client_command_lines.insert(id, "".to_string());
        }

        // create shared data
        let data_ref = Arc::new(Mutex::new(SimulationData::new(
            logs,
            stats,
            cc.egui_ctx.clone(),
        )));

        // spawn threads
        let drone_receiver = sc.get_drone_recv();
        let client_receiver = sc.get_client_recv();
        let server_receiver = sc.get_server_recv();
        let tmp_clone = Arc::clone(&data_ref);
        std::thread::spawn(move || {
            drone_receiver_loop(tmp_clone, drone_receiver);
        });
        let tmp_clone = Arc::clone(&data_ref);

        std::thread::spawn(move || {
            client_receiver_loop(tmp_clone, client_receiver);
        });
        let tmp_clone = Arc::clone(&data_ref);

        std::thread::spawn(move || {
            server_receiver_loop(tmp_clone, server_receiver);
        });

        // return
        Self {
            sc,
            simulation_data_ref: data_ref,
            open_windows,
            client_command_lines,
            drone_pdrs,
            add_link_selected_ids,
        }
    }

    pub fn sidebar(&mut self, ctx: &Context) {
        SidePanel::left("left").show(ctx, |ui| {
            ui.heading("Nodes");
            ui.separator();

            ui.label("Clients");
            ui.indent("clients", |ui| {
                // For every item, show its name as a clickable label.
                for id in self.sc.get_client_ids() {
                    self.spawn_node_list_element(ui, id, "Client");
                }
            });
            ui.separator();

            ui.label("Servers");
            ui.indent("servers", |ui| {
                // For every item, show its name as a clickable label.
                for id in self.sc.get_server_ids() {
                    self.spawn_node_list_element(ui, id, "Server");
                }
            });
            ui.separator();

            ui.label("Drones");
            ui.indent("drones", |ui| {
                // clickable labels
                for id in self.sc.get_drone_ids() {
                    self.spawn_node_list_element(ui, id, "Drone");
                }
            });
            ui.separator();

            // Quit button, will close all windows
            if ui.button("Quit").clicked() {
                std::process::exit(0);
            }
        });
    }

    pub fn client_window(&mut self, ctx: &Context, id: NodeId) {
        let open = self.open_windows.get_mut(&id).unwrap();
        Window::new(format!("Client #{}", id))
            .open(open)
            .min_size(vec2(200.0, 300.0))
            .max_size(vec2(500.0, 300.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // logs
                    Self::spawn_logs(ui, Arc::clone(&self.simulation_data_ref), id);

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        if ui.button("Send Fragment").clicked() {
                            self.sc.send_fragment_fair(id);
                        }
                        if ui.button("Send FloodRequest").clicked() {
                            self.sc.send_flood_request_fair(id);
                        }

                        /* command line
                        let line = self.client_command_lines.get_mut(&id).unwrap();
                        let command_line_response = ui.add(
                            TextEdit::singleline(line)
                                .desired_width(f32::INFINITY)
                                .font(TextStyle::Monospace),
                        );
                        if command_line_response.lost_focus()
                            && ui.input(|i| i.key_pressed(Key::Enter))
                        {
                            //log.push_str(format!("\n{}", line).as_str());
                            line.clear();
                            command_line_response.request_focus();
                        }
                        */
                    });
                });
            });
    }

    pub fn server_window(&mut self, ctx: &Context, id: NodeId) {
        let open = self.open_windows.get_mut(&id).unwrap();
        Window::new(format!("Server #{}", id))
            .open(open)
            //.default_open(false)
            .min_size(vec2(200.0, 300.0))
            .max_size(vec2(500.0, 300.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Stats about the server");
                });

                ui.add_space(5.0);

                ui.vertical(|ui| {
                    // logs
                    Self::spawn_logs(ui, Arc::clone(&self.simulation_data_ref), id);
                });
            });
    }

    pub fn drone_window(&mut self, ctx: &Context, id: NodeId) {
        let open = self.open_windows.get_mut(&id).unwrap();
        Window::new(format!("Drone #{}", id))
            .open(open)
            .fixed_size(vec2(400.0, 300.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // ----- stats -----
                    Self::spawn_drone_stats(ui, Arc::clone(&self.simulation_data_ref), id);
                    ui.add_space(5.0);

                    // ----- logs -----
                    Self::spawn_logs(ui, Arc::clone(&self.simulation_data_ref), id);
                    ui.add_space(5.0);

                    Self::spawn_white_heading(ui, "Actions");
                    ui.add_space(5.0);

                    // ----- actions -----

                    // TODO: show only not neighbor nodes
                    let mut node_ids = self.sc.get_drone_ids();
                    node_ids.append(&mut self.sc.get_client_ids());
                    node_ids.append(&mut self.sc.get_server_ids());
                    node_ids.sort();
                    let selected_id = self.add_link_selected_ids.get_mut(&id).unwrap();

                    ui.horizontal(|ui| {
                        ui.monospace("Add link with:");
                        ComboBox::from_label("")
                            .width(50.0)
                            .selected_text(
                                selected_id
                                    .map(|num| num.to_string())
                                    .unwrap_or_else(|| "-".to_string()),
                            )
                            .show_ui(ui, |ui| {
                                for &number in &node_ids {
                                    ui.selectable_value(selected_id, Some(number), number.to_string());
                                }
                            });
                        if ui.button("Add").clicked() {
                            let mut log_line = String::new();
                            match selected_id {
                                None => {
                                    log_line = "Error: id not selected".to_string()
                                }
                                Some(sid) => {
                                    log_line = match self.sc.add_edge(id, *sid) {
                                        Some(_) => {
                                            // push log to other node as well
                                            Self::push_log(
                                                Arc::clone(&self.simulation_data_ref),
                                                *sid,
                                                format!("Link added with node {}", id),
                                            );
                                            format!("Link added with node {}", *sid)
                                        }
                                        None => format!("Failed to add link with node {}", *sid),
                                    };
                                }
                            }

                            Self::push_log(
                                Arc::clone(&self.simulation_data_ref),
                                id,
                                log_line
                            );
                        }
                    });

                    ui.add_space(3.0);

                    ui.horizontal(|ui| {
                        ui.monospace("PDR:");
                        let response = ui.add(Slider::new(
                            self.drone_pdrs.get_mut(&id).unwrap(),
                            0.0..=1.0,
                        ));
                        if response.drag_stopped() {
                            let new_pdr: f32 = *self.drone_pdrs.get(&id).unwrap();
                            let log_line = match self.sc.set_pdr(id, new_pdr) {
                                Some(_) => format!("Changed PDR to {}", new_pdr),
                                None => "Failed to change PDR".to_string(),
                            };
                            Self::push_log(
                                Arc::clone(&self.simulation_data_ref),
                                id,
                                log_line
                            );

                        }
                    });

                    ui.add_space(3.0);

                    ui.horizontal(|ui| {
                        if ui.button("Crash").clicked() {
                            if let None = self.sc.crash_drone(id) {
                                Self::push_log(
                                    Arc::clone(&self.simulation_data_ref),
                                    id,
                                    "Failed to crash".to_string()
                                );
                            };
                        }
                        if ui.button("Clear log").clicked() {
                            let mut data = self.simulation_data_ref.lock().unwrap();
                            let v = data.logs.get_mut(&id).unwrap();
                            v.clear();
                        }
                    });
                });
            });
    }

    fn spawn_logs(ui: &mut Ui, m: Arc<Mutex<SimulationData>>, id: NodeId) {
        Self::spawn_white_heading(ui, "History");
        ui.add_space(5.0);
        ui.group(|ui| {
            ScrollArea::vertical()
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let v = m.lock().unwrap();
                    let v = v.logs.get(&id).unwrap();
                    for line in v {
                        ui.monospace(line);
                    }
                });
        });
    }

    fn push_log(m: Arc<Mutex<SimulationData>>, id: NodeId, line: String) {
        let mut data = m.lock().unwrap();
        let v = data.logs.get_mut(&id).unwrap();
        v.push(line);
    }

    fn spawn_drone_stats(ui: &mut Ui, m: Arc<Mutex<SimulationData>>, id: NodeId) {
        let data = m.lock().unwrap();
        let stats = data.stats.get(&id).unwrap();
        Self::spawn_white_heading(ui, "Statistics");
        Grid::new("done_stats").striped(true).show(ui, |ui| {
            // First row
            for header in [
                "Packet type ",
                "Fragment",
                "Ack",
                "Nack",
                "Flood Req.",
                "Flood Resp.",
            ] {
                ui.with_layout(
                    Layout::centered_and_justified(Direction::LeftToRight),
                    |ui| {
                        // let bold_monospace_text =
                        //     RichText::new(header).monospace().color(Color32::WHITE);
                        // ui.label(bold_monospace_text);
                        ui.monospace(header);
                    },
                );
            }
            ui.end_row();

            // Second row
            ui.with_layout(
                Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    // let bold_monospace_text =
                    //     RichText::new("Forwarded").monospace().color(Color32::WHITE);
                    // ui.label(bold_monospace_text);
                    ui.monospace("Forwarded");
                },
            );
            for n in stats.packets_forwarded {
                ui.with_layout(
                    Layout::centered_and_justified(Direction::LeftToRight),
                    |ui| {
                        ui.monospace(n.to_string());
                    },
                );
            }
            ui.end_row();
        });

        ui.add_space(5.0);

        ui.monospace(format!("Fragments dropped: {}", stats.fragments_dropped));
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

    fn spawn_white_heading(ui: &mut Ui, str: &'static str) {
        let text =
            RichText::new(str).monospace().color(Color32::WHITE);
        ui.heading(text);
    }
}
