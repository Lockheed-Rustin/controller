use eframe::egui::{vec2, Context, Image, ScrollArea, Window};
use crate::app::simulation_controller_ui::{ContentFile, ContentFileType};

pub fn spawn_file_window(
    ctx: &Context,
    open: &mut bool,
    state: &mut ContentFile,
) {
    Window::new(&state.name)
        .open(open)
        .min_size(vec2(100.0, 100.0))
        .max_size(vec2(300.0, 200.0))
        .show(ctx, |ui| {
            match &state.file {
                ContentFileType::Image(img) => {
                    ui.centered_and_justified(|ui| {
                        ui.add(Image::new(img).max_width(200.0).rounding(10.0));
                    });
                }
                ContentFileType::Text(s) => {
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.label(s);
                        });
                }
            }

        });
}


