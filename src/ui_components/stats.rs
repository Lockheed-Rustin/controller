use std::sync::MutexGuard;

use eframe::egui::{Direction, Grid, Layout, Ui};

use wg_2024::network::NodeId;

use crate::shared_data::SimulationData;
use crate::ui_components::text::spawn_white_heading;

/// spawns stats section for drone with given id
pub fn spawn_drone(ui: &mut Ui, mutex: &MutexGuard<SimulationData>, id: NodeId) {
    let stats = mutex.drone_stats.get(&id).unwrap();
    spawn_white_heading(ui, "Statistics");
    Grid::new("done_stats").striped(true).show(ui, |ui| {
        // First row
        spawn_packet_stats_table_header(ui);

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

/// spawns stats section for drone with given id
pub fn spawn_client(ui: &mut Ui, mutex: &MutexGuard<SimulationData>, id: NodeId) {
    let stats = mutex.client_stats.get(&id).unwrap();
    spawn_white_heading(ui, "Statistics");
    Grid::new("client_stats").striped(true).show(ui, |ui| {
        // First row
        spawn_packet_stats_table_header(ui);

        // Second row
        ui.with_layout(
            Layout::centered_and_justified(Direction::LeftToRight),
            |ui| {
                ui.monospace("Sent");
            },
        );
        for n in stats.packets_sent {
            ui.with_layout(
                Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.monospace(n.to_string());
                },
            );
        }
        ui.end_row();

        // Third row
        ui.with_layout(
            Layout::centered_and_justified(Direction::LeftToRight),
            |ui| {
                ui.monospace("Received");
            },
        );
        for n in stats.packets_received {
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

    ui.monospace(format!(
        "Fragmented messages: {}   Assembled messages: {}",
        stats.messages_fragmented, stats.messages_assembled
    ));

    ui.add_space(2.0);
    ui.separator();
}

/// spawns server section for drone with given id
pub fn spawn_server(ui: &mut Ui, mutex: &MutexGuard<SimulationData>, id: NodeId) {
    let stats = mutex.server_stats.get(&id).unwrap();
    spawn_white_heading(ui, "Statistics");
    Grid::new("server_stats").striped(true).show(ui, |ui| {
        // First row
        spawn_packet_stats_table_header(ui);

        // Second row
        ui.with_layout(
            Layout::centered_and_justified(Direction::LeftToRight),
            |ui| {
                ui.monospace("Sent");
            },
        );
        for n in stats.packets_sent {
            ui.with_layout(
                Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.monospace(n.to_string());
                },
            );
        }
        ui.end_row();

        // Third row
        ui.with_layout(
            Layout::centered_and_justified(Direction::LeftToRight),
            |ui| {
                ui.monospace("Received");
            },
        );
        for n in stats.packets_received {
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

    ui.monospace(format!(
        "Fragmented messages: {}   Assembled messages: {}",
        stats.messages_fragmented, stats.messages_assembled
    ));

    ui.add_space(2.0);
    ui.separator();
}

fn spawn_packet_stats_table_header(ui: &mut Ui) {
    for header in [
        "Packet type",
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
}
