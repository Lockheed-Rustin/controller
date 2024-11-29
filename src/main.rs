mod simulation_controller_ui;

use std::fs;
use eframe::egui;
use wg_2024::config::Config;
use drone_networks::network::*;
use simulation_controller_ui::SimulationControllerUI;


fn parse_config(file: &str) -> Config {
    let file_str = fs::read_to_string(file).unwrap();
    toml::from_str(&file_str).unwrap()
}

fn main() -> eframe::Result {
    let file_str = fs::read_to_string("config.toml").unwrap();
    let config = toml::from_str(&file_str).unwrap();

    let sm = init_network(&config).unwrap();
    let sm_ui = Box::new(SimulationControllerUI::new(sm));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((600.0, 600.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Simulation Controller",
        native_options,
        Box::new(|_| Ok(sm_ui)),
    )
}
