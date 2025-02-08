use crate::app::{ContentFile, ContentFileType};
use crate::ui_components;
use eframe::egui::{vec2, Context, Image, Window};

pub fn spawn_file_window(
    ctx: &Context,
    open: &mut bool,
    state: &mut ContentFile,
) {
    Window::new(&state.name)
        .open(open)
        .fixed_size(vec2(400.0, 300.0))
        .show(ctx, |ui| {
            match &state.file {
                ContentFileType::Image(img) => {
                    ui.add(Image::new(img).max_width(200.0).rounding(10.0));
                }
                ContentFileType::Text(s) => {
                    ui.label(s);
                }
            }

        });
}


