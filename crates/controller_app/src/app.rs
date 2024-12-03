use eframe::egui::{
    vec2, CentralPanel, Context, CursorIcon, Key, Label, ScrollArea, Sense, SidePanel, TextEdit,
    TextStyle, Ui, Window,
};
use eframe::CreationContext;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use drone_networks::controller::SimulationController;
use wg_2024::controller::DroneCommand;
use wg_2024::network::NodeId;

use controller_data::{SimulationData, DroneStats};
use controller_receiver_thread::receiver_loop;

pub struct SimulationControllerUI {
    sc: SimulationController,
    simulation_data_ref: Arc<Mutex<SimulationData>>,
    open_windows: HashMap<NodeId, bool>,
    clients_command_lines: HashMap<NodeId, String>,
}

impl eframe::App for SimulationControllerUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // sidebar
        self.sidebar(ctx);
        // node windows
        CentralPanel::default().show(ctx, |ui| {
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

        // create hashmaps
        let mut h_str = HashMap::new();
        let mut h_bool = HashMap::new();
        for id in ids.clone() {
            h_str.insert(id, "".to_string());
            h_bool.insert(id, false);
        }

        // drone stat hashmap
        let mut h_stats = HashMap::new();
        for id in sc.get_drone_ids() {
            h_stats.insert(id, DroneStats::default());
        }

        // create shared data and spawn thread
        let data_ref = Arc::new(Mutex::new(SimulationData::new(
            h_str.clone(),
            h_stats,
            cc.egui_ctx.clone(),
        )));
        let data_ref_clone = Arc::clone(&data_ref);
        let sc_receiver_clone = sc.get_receiver();
        std::thread::spawn(move || {
            receiver_loop(data_ref_clone, sc_receiver_clone);
        });

        Self {
            sc,
            simulation_data_ref: data_ref,
            open_windows: h_bool,
            clients_command_lines: h_str,
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
        let line = self.clients_command_lines.get_mut(&id).unwrap();
        //let log = self.logs.get_mut(&id).unwrap();
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
                                //ui.monospace(log.clone());
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
                            //log.push_str(format!("\n{}", line).as_str());
                            line.clear();
                            command_line_response.request_focus();
                        }
                    });
                });
            });
    }

    pub fn server_window(&mut self, ctx: &Context, id: NodeId) {
        //let log = self.logs.get_mut(&id).unwrap();
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
                                //ui.monospace(log.clone());
                            });
                    });
                });
            });
    }

    pub fn drone_window(&mut self, ctx: &Context, id: NodeId) {
        //let log = self.logs.get_mut(&id).unwrap();
        let open = self.open_windows.get_mut(&id).unwrap();
        //let sender = self.sm.;

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
                                //ui.monospace(log.clone());
                            });
                    });

                    ui.add_space(5.0);

                    // Bottom panel replacement
                    /*
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
                     */
                });
            });
    }

    fn get_node_ids(&self) -> Vec<NodeId> {
        // let mut res = self.sm.get_server_ids();
        // res.append(&mut self.sm.get_drone_ids());
        // res.append(&mut self.sm.get_client_ids());
        // res
        vec![]
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
