use crate::data::SimulationData;
use crate::ui_components;
use eframe::egui::{vec2, Context, Window};
use std::sync::MutexGuard;
use wg_2024::network::NodeId;

pub fn spawn_server_window(
    ctx: &Context,
    mut mutex: MutexGuard<SimulationData>,
    open: &mut bool,
    id: NodeId,
) {
    Window::new(format!("Server #{}", id))
        .open(open)
        .fixed_size(vec2(400.0, 300.0))
        .show(ctx, |ui| {
            ui.add_space(5.0);

            ui.vertical(|ui| {
                // logs
                ui_components::logs::spawn_logs(ui, &mutex, id);
            });

            ui_components::text::spawn_white_heading(ui, "Actions");
            ui.add_space(5.0);

            if ui.button("Clear log").clicked() {
                let v = mutex.logs.get_mut(&id).unwrap();
                v.clear();
            }
        });
}
