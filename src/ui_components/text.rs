use eframe::egui::{Color32, RichText, Ui};

/// Spawns a white monospace heading
pub fn spawn_white_heading(ui: &mut Ui, str: &'static str) {
    let text = RichText::new(str).monospace().color(Color32::WHITE);
    ui.heading(text);
}
