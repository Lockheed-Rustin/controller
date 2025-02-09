use std::sync::MutexGuard;

use eframe::egui::{Color32, RichText, ScrollArea, Ui};

use wg_2024::network::NodeId;

use crate::data::SimulationData;
use crate::ui_components::text::spawn_white_heading;

pub fn spawn(ui: &mut Ui, mutex: &MutexGuard<SimulationData>, id: NodeId) {
    spawn_white_heading(ui, "History");
    ui.add_space(5.0);
    ui.group(|ui| {
        ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for line in mutex.get_logs(id) {
                    ui.label(colored_monospace_text(&line.0, line.1));
                }
            });
    });
}

fn colored_monospace_text(text: &String, color: Color32) -> RichText {
    RichText::new(text).monospace().color(color)
}
