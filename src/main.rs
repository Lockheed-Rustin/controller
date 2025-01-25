use drone_networks::network::*;
use eframe::egui;
use std::fs;

mod app;
pub mod data;
mod receiver_threads;
mod ui_components;

fn main() -> eframe::Result {
    // read config file and get a SimulationController
    let file_str = fs::read_to_string("config.toml").unwrap();
    let config = toml::from_str(&file_str).unwrap();
    let sc = init_network(&config).unwrap();

    // window options
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((1000.0, 700.0)),
        ..eframe::NativeOptions::default()
    };

    // run ui
    eframe::run_native(
        "Simulation Controller",
        native_options,
        Box::new(|cc| Ok(Box::new(app::SimulationControllerUI::new(cc, sc)))),
    )
}
