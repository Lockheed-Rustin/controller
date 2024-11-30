use std::collections::{HashMap, HashSet};
use eframe::egui::{
    vec2, CentralPanel, Context, CursorIcon, Key, Label, ScrollArea, Sense, SidePanel, TextEdit,
    TextStyle, Ui, Window,
};
use drone_networks::controller::SimulationController;
use wg_2024::controller::DroneCommand;
use wg_2024::network::NodeId;

pub struct SimulationControllerUI {
    sm: SimulationController,
    logs: HashMap<NodeId, String>,
    open_windows: HashMap<NodeId, bool>,
    clients_command_lines: HashMap<NodeId, String>,
}

impl eframe::App for SimulationControllerUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        SidePanel::left("left").show(ctx, |ui| {
            ui.heading("Nodes");
            ui.separator();

            ui.label("Clients");
            ui.indent("clients", |ui| {
                // For every item, show its name as a clickable label.
                for id in self.get_client_ids() {
                    self.spawn_node_list_element(ui, id, "Client");
                }
            });
            ui.separator();

            ui.label("Servers");
            ui.indent("servers", |ui| {
                // For every item, show its name as a clickable label.
                for id in self.get_server_ids() {
                    self.spawn_node_list_element(ui, id, "Server");
                }
            });
            ui.separator();

            ui.label("Drones");
            ui.indent("drones", |ui| {
                // clickable labels
                for id in self.get_drone_ids() {
                    self.spawn_node_list_element(ui, id, "Drone");
                }
            });
            ui.separator();

            // Quit button, will close all windows
            if ui.button("Quit").clicked() {
                std::process::exit(0);
            }
        });

        CentralPanel::default().show(ctx, |ui| {
            for id in self.get_drone_ids() {
                self.drone_window(ctx, id);
            }
            for id in self.get_server_ids() {
                self.server_window(ctx, id);
            }
            for id in self.get_client_ids() {
                self.client_window(ctx, id);
            }
        });
    }
}

impl SimulationControllerUI {
    pub fn new(sm: SimulationController) -> Self {
        // TODO: Add topology!

        // get all node ids
        let mut ids = HashSet::new();
        for client_id in sm.clients_send.keys() {
            ids.insert(*client_id);
        }
        for drone_id in sm.drones_send.keys() {
            ids.insert(*drone_id);
        }
        for server_id in sm.server_ids.iter() {
            ids.insert(*server_id);
        }

        // create hashmaps
        let mut h_str = HashMap::new();
        let mut h_bool = HashMap::new();
        for id in ids.clone() {
            h_str.insert(id, "".to_string());
            h_bool.insert(id, false);
        }

        Self {
            sm,
            logs: h_str.clone(),
            open_windows: h_bool,
            clients_command_lines: h_str.clone(),
        }
    }

    pub fn client_window(&mut self, ctx: &Context, id: NodeId) {
        let line = self.clients_command_lines.get_mut(&id).unwrap();
        let log = self.logs.get_mut(&id).unwrap();
        let open = self.open_windows.get_mut(&id).unwrap();

        Window::new(format!("Client #{}", id))
            .open(open) // Automatically closes when X is clicked
            //.default_open(false)
            .min_size(vec2(200.0, 300.0)) // Minimum dimensions
            .max_size(vec2(500.0, 300.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Central panel replacement
                    ui.group(|ui| {
                        ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.monospace(log.clone());
                            });
                    });

                    ui.add_space(5.0); // Add some spacing between the sections

                    // Bottom panel replacement
                    ui.horizontal(|ui| {
                        let command_line_response = ui.add(
                            TextEdit::singleline(line)
                                .desired_width(f32::INFINITY)
                                .font(TextStyle::Monospace),
                        );

                        if command_line_response.lost_focus()
                            && ui.input(|i| i.key_pressed(Key::Enter))
                        {
                            log.push_str(format!("\n{}", line).as_str());
                            line.clear();
                            command_line_response.request_focus();
                        }
                    });
                });
            });
    }

    pub fn server_window(&mut self, ctx: &Context, id: NodeId) {
        let log = self.logs.get_mut(&id).unwrap();
        let open = self.open_windows.get_mut(&id).unwrap();

        Window::new(format!("Server #{}", id))
            .open(open) // Automatically closes when X is clicked
            //.default_open(false)
            .min_size(vec2(200.0, 300.0)) // Minimum dimensions
            .max_size(vec2(500.0, 300.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Stats about the server");
                });

                ui.add_space(5.0); // Add some spacing between the sections

                ui.vertical(|ui| {
                    // Central panel replacement
                    ui.group(|ui| {
                        ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.monospace(log.clone());
                            });
                    });
                });
            });
    }

    pub fn drone_window(&mut self, ctx: &Context, id: NodeId) {
        let log = self.logs.get_mut(&id).unwrap();
        let open = self.open_windows.get_mut(&id).unwrap();
        let sender = self.sm.drones_send.get(&id).unwrap();

        Window::new(format!("Drone #{}", id))
            .open(open) // Automatically closes when X is clicked
            .min_size(vec2(200.0, 300.0)) // Minimum dimensions
            .max_size(vec2(500.0, 300.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Central panel replacement
                    ui.group(|ui| {
                        ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.monospace(log.clone());
                            });
                    });

                    ui.add_space(5.0);

                    // Bottom panel replacement
                    ui.horizontal(|ui| {
                        if ui.button("Crash").clicked() {
                            match sender.send(DroneCommand::Crash) {
                                Ok(_) => {
                                    log.push_str("drone crashed successfully\n");
                                }
                                Err(_) => log.push_str("ERROR: drone already crashed\n"),
                            }
                        }
                        if ui.button("Add link").clicked() {
                            // TODO: change message
                            match sender.send(DroneCommand::SetPacketDropRate(69.69)) {
                                Ok(_) => {
                                    log.push_str(&format!("added link with node #{}\n", id));
                                }
                                Err(e) => {
                                    log.push_str(&format!("ERROR: {}\n", e));
                                }
                            }
                        }
                        if ui.button("Change PDR").clicked() {
                            // TODO: change message
                            let tmp = 69.69;
                            match sender.send(DroneCommand::SetPacketDropRate(tmp)) {
                                Ok(_) => {
                                    log.push_str(&format!("changed PDR to {}\n", tmp));
                                }
                                Err(e) => {
                                    log.push_str(&format!("ERROR: {}\n", e));
                                }
                            }
                        }
                    });
                });
            });
    }

    fn get_node_ids(&self) -> Vec<NodeId> {
        let mut res = self.get_server_ids();
        res.append(&mut self.get_drone_ids());
        res.append(&mut self.get_client_ids());
        res
    }

    fn get_client_ids(&self) -> Vec<NodeId> {
        let mut res = vec![];
        for id in self.sm.clients_send.keys() {
            res.push(*id);
        }
        res
    }

    fn get_drone_ids(&self) -> Vec<NodeId> {
        let mut res = vec![];
        for id in self.sm.drones_send.keys() {
            res.push(*id);
        }
        res
    }

    fn get_server_ids(&self) -> Vec<NodeId> {
        self.sm.server_ids.clone()
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
}
