use crate::data::SimulationData;
use crate::ui_components;
use eframe::egui::{vec2, ComboBox, Context, Slider, Window};
use std::sync::MutexGuard;
use wg_2024::network::NodeId;

pub fn spawn_drone_window(
    ctx: &Context,
    mut mutex: MutexGuard<SimulationData>,
    open: &mut bool,                  // window's state
    id: NodeId,                       // drone id
    node_ids: Vec<NodeId>,            // all other nodes
    selected_id: &mut Option<NodeId>, // "add link" form state
    pdr_slider: &mut f32,             // pdr slider state
) {
    Window::new(format!("Drone #{}", id))
        .open(open)
        .fixed_size(vec2(400.0, 300.0))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // ----- stats -----
                ui_components::stats::spawn_drone_stats(ui, &mutex, id);
                ui.add_space(5.0);

                // ----- logs -----
                ui_components::logs::spawn_logs(ui, &mutex, id);
                ui.add_space(5.0);

                ui_components::text::spawn_white_heading(ui, "Actions");
                ui.add_space(5.0);

                // ----- actions -----
                //let selected_id = self.add_link_selected_ids.get_mut(&id).unwrap();

                ui.horizontal(|ui| {
                    ui.monospace("Add link with:");
                    ComboBox::from_id_salt("combobox")
                        .width(50.0)
                        .selected_text(
                            selected_id
                                .map(|num| num.to_string())
                                .unwrap_or_else(|| "-".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            for number in node_ids {
                                ui.selectable_value(selected_id, Some(number), number.to_string());
                            }
                        });
                    if ui.button("Add").clicked() {
                        let log_line = match selected_id {
                            None => "Error: id not selected".to_string(),
                            Some(sid) => {
                                println!("trying add {} and {}", id, *sid);
                                match mutex.sc.add_edge(id, *sid) {
                                    Some(_) => {
                                        // push log to other node as well
                                        push_log(
                                            &mut mutex,
                                            *sid,
                                            format!("Link added with node {}", id),
                                        );
                                        format!("Link added with node {}", *sid)
                                    }
                                    None => format!("Failed to add link with node {}", *sid),
                                }
                            }
                        };

                        push_log(&mut mutex, id, log_line);
                    }
                });

                ui.add_space(3.0);

                ui.horizontal(|ui| {
                    ui.monospace("PDR:");
                    let response = ui.add(Slider::new(
                        pdr_slider,
                        //self.drone_pdr_sliders.get_mut(&id).unwrap(),
                        0.0..=1.0,
                    ));
                    if response.drag_stopped() {
                        let log_line = match mutex.sc.set_pdr(id, *pdr_slider) {
                            Some(_) => format!("Changed PDR to {}", pdr_slider),
                            None => "Failed to change PDR".to_string(),
                        };
                        push_log(&mut mutex, id, log_line);
                    }
                });

                ui.add_space(3.0);

                ui.horizontal(|ui| {
                    if ui.button("Crash").clicked() && mutex.sc.crash_drone(id).is_none() {
                        push_log(&mut mutex, id, "Failed to crash".to_string());
                    }
                    if ui.button("Clear log").clicked() {
                        let v = mutex.logs.get_mut(&id).unwrap();
                        v.clear();
                    }
                });
            });
        });
}

fn push_log(mutex: &mut MutexGuard<SimulationData>, id: NodeId, line: String) {
    let v = mutex.logs.get_mut(&id).unwrap();
    v.push(line);
}
