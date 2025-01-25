use eframe::egui::{Direction, Grid, Layout, Ui};
use std::sync::MutexGuard;
use wg_2024::network::NodeId;

use crate::data::SimulationData;
use crate::ui_components::text::spawn_white_heading;

pub fn spawn_drone_stats(ui: &mut Ui, mutex: &MutexGuard<SimulationData>, id: NodeId) {
    let stats = mutex.stats.get(&id).unwrap();
    spawn_white_heading(ui, "Statistics");
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
                    ui.monospace(header);
                },
            );
        }
        ui.end_row();

        // Second row
        ui.with_layout(
            Layout::centered_and_justified(Direction::LeftToRight),
            |ui| {
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
