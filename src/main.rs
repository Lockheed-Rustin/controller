use app::simulation_controller_ui;
use eframe::egui;

mod app;
pub mod data;
mod receiver_threads;
mod ui_components;

fn main() -> eframe::Result {
    // window options
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((1000.0, 700.0)),
        ..eframe::NativeOptions::default()
    };

    // run ui
    eframe::run_native(
        "Simulation Controller",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(
                simulation_controller_ui::SimulationControllerUI::new(cc),
            ))
        }),
    )
}
