use std::sync::MutexGuard;

use eframe::egui::{ScrollArea, Ui};

use wg_2024::network::NodeId;

use crate::data::SimulationData;
use crate::ui_components::text::spawn_white_heading;

pub fn spawn_logs(ui: &mut Ui, mutex: &MutexGuard<SimulationData>, id: NodeId) {
    spawn_white_heading(ui, "History");
    ui.add_space(5.0);
    ui.group(|ui| {
        ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let v = mutex.logs.get(&id).unwrap();
                for line in v {
                    ui.monospace(line);
                }
            });
    });
}
