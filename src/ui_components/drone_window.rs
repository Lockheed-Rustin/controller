use std::sync::MutexGuard;

use eframe::egui::{vec2, Color32, ComboBox, Context, Slider, Window};

use crate::app::DroneWindowState;
use crate::data::SimulationData;
use crate::ui_components;
use wg_2024::network::NodeId;

pub fn spawn_drone_window(
    ctx: &Context,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,            // drone id
    node_ids: Vec<NodeId>, // all other nodes
    open: &mut bool,
    state: &mut DroneWindowState,
) {
    Window::new(format!("Drone #{}", id))
        .open(open)
        .fixed_size(vec2(400.0, 300.0))
        .show(ctx, |ui| {
            // ----- stats -----
            ui_components::stats::spawn_drone_stats(ui, mutex, id);
            ui.add_space(5.0);

            // ----- logs -----
            ui_components::logs::spawn_logs(ui, mutex, id);
            ui.add_space(5.0);

            ui_components::text::spawn_white_heading(ui, "Actions");
            ui.add_space(5.0);

            // ----- actions -----
            ui.horizontal(|ui| {
                ui.monospace("Add link with:");
                ComboBox::from_id_salt("combobox")
                    .width(50.0)
                    .selected_text(
                        state
                            .add_link_selected_id
                            .map_or_else(|| "-".to_string(), |num| num.to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for number in node_ids {
                            ui.selectable_value(
                                &mut state.add_link_selected_id,
                                Some(number),
                                number.to_string(),
                            );
                        }
                    });
                if ui.button("Add").clicked() {
                    let log_line = match state.add_link_selected_id {
                        None => "Error: id not selected".to_string(),
                        Some(sid) => {
                            match mutex.sc.add_edge(id, sid) {
                                Some(_) => {
                                    // push log to other node as well
                                    mutex.add_log(
                                        sid,
                                        format!("Link added with node {id}"),
                                        Color32::WHITE,
                                    );
                                    format!("Link added with node {sid}")
                                }
                                None => format!("Failed to add link with node {sid}"),
                            }
                        }
                    };

                    mutex.add_log(id, log_line, Color32::WHITE);
                }
            });

            ui.add_space(3.0);

            ui.horizontal(|ui| {
                ui.monospace("PDR:");
                let response = ui.add(Slider::new(
                    &mut state.pdr_slider,
                    //self.drone_pdr_sliders.get_mut(&id).unwrap(),
                    0.0..=1.0,
                ));
                if response.drag_stopped() || response.lost_focus() {
                    let log_line = match mutex.sc.set_pdr(id, state.pdr_slider) {
                        Some(_) => format!("Changed PDR to {}", state.pdr_slider),
                        None => "Failed to change PDR".to_string(),
                    };
                    mutex.add_log(id, log_line, Color32::WHITE);
                }
            });

            ui.add_space(3.0);

            ui.horizontal(|ui| {
                if ui.button("Crash").clicked() && mutex.sc.crash_drone(id).is_none() {
                    mutex.add_log(id, "Cannot crash".to_string(), Color32::LIGHT_RED);
                }
                if ui.button("Clear log").clicked() {
                    mutex.clear_log(id);
                }
            });
        });
}
