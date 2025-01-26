use std::sync::MutexGuard;

use eframe::egui::{vec2, Context, Window};

use wg_2024::network::NodeId;

use crate::data::SimulationData;
use crate::ui_components;

pub fn spawn_client_window(
    ctx: &Context,
    mut mutex: MutexGuard<SimulationData>,
    open: &mut bool,
    id: NodeId,
) {
    Window::new(format!("Client #{}", id))
        .open(open)
        .fixed_size(vec2(400.0, 300.0))
        .show(ctx, |ui| {
            // logs
            ui_components::logs::spawn_logs(ui, &mutex, id);
            ui.add_space(5.0);

            // actions
            ui_components::text::spawn_white_heading(ui, "Actions");
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if ui.button("Send Fragment").clicked() {
                    mutex.sc.send_fragment_fair(id);
                }
                if ui.button("Send Ack").clicked() {
                    mutex.sc.send_ack_fair(id);
                }
                if ui.button("Send FloodRequest").clicked() {
                    mutex.sc.send_flood_request_fair(id);
                }
                if ui.button("Clear log").clicked() {
                    let v = mutex.logs.get_mut(&id).unwrap();
                    v.clear();
                }

                /* command line
                let line = self.client_command_lines.get_mut(&id).unwrap();
                let command_line_response = ui.add(
                    TextEdit::singleline(line)
                        .desired_width(f32::INFINITY)
                        .font(TextStyle::Monospace),
                );
                if command_line_response.lost_focus()
                    && ui.input(|i| i.key_pressed(Key::Enter))
                {
                    //log.push_str(format!("\n{}", line).as_str());
                    line.clear();
                    command_line_response.request_focus();
                }
                */
            });
        });
}
