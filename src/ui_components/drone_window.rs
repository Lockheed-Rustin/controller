use std::sync::MutexGuard;

use eframe::egui::{vec2, Color32, ComboBox, Context, Slider, Ui, Window};

use crate::app::simulation_controller_ui::DroneWindowState;
use crate::shared_data::SimulationData;
use crate::ui_components;
use wg_2024::network::NodeId;

/// Spawns the drone window.
/// #Arguments
/// `id` is the id of the drone whose window needs to be spawned.
pub fn spawn(
    ctx: &Context,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    node_ids: &[NodeId],
    open: &mut bool,
    state: &mut DroneWindowState,
) {
    Window::new(format!("{} #{id}", state.name))
        .open(open)
        .fixed_size(vec2(400.0, 300.0))
        .show(ctx, |ui| {
            // ----- stats -----
            ui_components::stats::spawn_drone(ui, mutex, id);
            ui.add_space(5.0);

            // ----- logs -----
            ui_components::logs::spawn(ui, mutex, id);
            ui.add_space(5.0);

            ui_components::text::spawn_white_heading(ui, "Actions");
            ui.add_space(5.0);

            // ----- actions -----
            ui.horizontal(|ui| {
                ui.monospace("Add link with:");
                spawn_add_link_combobox(ui, id, node_ids, state);
                spawn_add_button(ui, mutex, id, state);
            });

            ui.add_space(3.0);

            ui.horizontal(|ui| {
                ui.monospace("PDR:");
                spawn_pdr_slider(ui, mutex, id, state);
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

/// Spawns the drop-down menu for choosing the node id to add a link with.
fn spawn_add_link_combobox(
    ui: &mut Ui,
    id: NodeId,
    node_ids: &[NodeId],
    state: &mut DroneWindowState,
) {
    ComboBox::from_id_salt("combobox")
        .width(50.0)
        .selected_text(
            state
                .add_link_selected_id
                .map_or_else(|| "-".to_string(), |num| num.to_string()),
        )
        .show_ui(ui, |ui| {
            for number in node_ids.iter().filter(|i| **i != id) {
                ui.selectable_value(
                    &mut state.add_link_selected_id,
                    Some(*number),
                    number.to_string(),
                );
            }
        });
}

/// Spawns the button for adding a link.
fn spawn_add_button(
    ui: &mut Ui,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    state: &mut DroneWindowState,
) {
    if ui.button("Add").clicked() {
        let log_line = match state.add_link_selected_id {
            None => "Error: id not selected".to_string(),
            Some(sid) => {
                match mutex.sc.add_edge(id, sid) {
                    Some(()) => {
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
}

/// Spawns the slider for changing the PDR
fn spawn_pdr_slider(
    ui: &mut Ui,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    state: &mut DroneWindowState,
) {
    let response = ui.add(Slider::new(
        &mut state.pdr_slider,
        0.0..=1.0,
    ));
    if response.drag_stopped() || response.lost_focus() {
        let log_line = match mutex.sc.set_pdr(id, state.pdr_slider) {
            Some(()) => format!("Changed PDR to {}", state.pdr_slider),
            None => "Failed to change PDR".to_string(),
        };
        mutex.add_log(id, log_line, Color32::WHITE);
    }
}
