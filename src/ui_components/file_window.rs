use crate::app::simulation_controller_ui::{ContentFile, ContentFileType};
use eframe::egui::{vec2, Context, Image, ScrollArea, Window};

/// Spawns a window containing a file.
pub fn spawn(ctx: &Context, open: &mut bool, state: &mut ContentFile) {
    Window::new(&state.name)
        .open(open)
        .min_size(vec2(100.0, 100.0))
        .max_size(vec2(250.0, 250.0))
        .show(ctx, |ui| match &state.file {
            ContentFileType::Image(img) => {
                ui.centered_and_justified(|ui| {
                    ui.add(
                        Image::new(img)
                            .fit_to_exact_size(vec2(200.0, 200.0))
                            .max_width(200.0)
                            .rounding(10.0),
                    );
                });
            }
            ContentFileType::Text(s) => {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.label(s);
                    });
            }
        });
}
