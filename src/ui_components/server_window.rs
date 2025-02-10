use std::sync::MutexGuard;

use eframe::egui::{vec2, Context, Window};

use wg_2024::network::NodeId;

use crate::shared_data::SimulationData;
use crate::ui_components;

/// Spawns the server window.
pub fn spawn(
    ctx: &Context,
    mutex: &mut MutexGuard<SimulationData>,
    open: &mut bool,
    id: NodeId,
) {
    Window::new(format!("Server #{id}"))
        .open(open)
        .fixed_size(vec2(400.0, 300.0))
        .show(ctx, |ui| {
            ui_components::stats::spawn_server(ui, mutex, id);
            // logs
            ui_components::logs::spawn(ui, mutex, id);

            ui_components::text::spawn_white_heading(ui, "Actions");
            ui.add_space(5.0);

            if ui.button("Clear log").clicked() {
                mutex.clear_log(id);
            }
        });
}
