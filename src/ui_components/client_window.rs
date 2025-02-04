use std::fmt::{Display, Formatter};
use std::sync::MutexGuard;

use eframe::egui::{vec2, ComboBox, Context, Ui, Window};

use wg_2024::network::NodeId;

use crate::data::SimulationData;
use crate::ui_components;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MessageChoice {
    NotChosen,
    ReqServerType,
    Content,
    Communication,
}

#[derive(PartialEq, Copy, Clone)]
pub enum ContentChoice {
    NotChosen,
    ReqFilesList,
    ReqFile,
}

#[derive(PartialEq, Copy, Clone)]
pub enum CommunicationChoice {
    NotChosen,
    ReqRegistrationToChat,
    MessageSend,
    ReqClientsList,
}

pub fn spawn_client_window(
    ctx: &Context,
    mut mutex: MutexGuard<SimulationData>,
    open: &mut bool,
    id: NodeId,
    message_choice: &mut MessageChoice,
    content_choice: &mut ContentChoice,
    communication_choice: &mut CommunicationChoice,
    text_input: &mut String,
) {
    Window::new(format!("Client #{}", id))
        .open(open)
        .fixed_size(vec2(400.0, 300.0))
        .show(ctx, |ui| {
            // logs
            ui_components::logs::spawn_logs(ui, &mutex, id);
            if ui.button("Clear log").clicked() {
                let v = mutex.logs.get_mut(&id).unwrap();
                v.clear();
            }
            ui.add_space(5.0);

            // actions
            ui_components::text::spawn_white_heading(ui, "Actions");
            ui.add_space(5.0);

            spawn_message_combobox(ui, message_choice, content_choice, communication_choice);

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
}

fn spawn_message_combobox(
    ui: &mut Ui,
    message_choice: &mut MessageChoice,
    content_choice: &mut ContentChoice,
    communication_choice: &mut CommunicationChoice,
) {
    ComboBox::from_id_salt("message-choice")
        .width(210.0)
        .selected_text(format!("{}", message_choice))
        .show_ui(ui, |ui| {
            spawn_choice(ui, message_choice, MessageChoice::ReqServerType);
            spawn_choice(ui, message_choice, MessageChoice::Content);
            spawn_choice(ui, message_choice, MessageChoice::Communication);
        });
    match message_choice {
        MessageChoice::NotChosen => {}
        MessageChoice::ReqServerType => {}
        MessageChoice::Content => {
            *communication_choice = CommunicationChoice::NotChosen;
            spawn_content_combobox(ui, content_choice);
        }
        MessageChoice::Communication => {
            *content_choice = ContentChoice::NotChosen;
            spawn_communication_combobox(ui, communication_choice);
        }
    }
}

fn spawn_content_combobox(ui: &mut Ui, content_choice: &mut ContentChoice) {
    ComboBox::from_id_salt("content-choice")
        .width(210.0)
        .selected_text(format!("{}", content_choice))
        .show_ui(ui, |ui| {
            spawn_choice(ui, content_choice, ContentChoice::ReqFile);
            spawn_choice(ui, content_choice, ContentChoice::ReqFilesList);
        });
}

fn spawn_communication_combobox(ui: &mut Ui, communication_choice: &mut CommunicationChoice) {
    ComboBox::from_id_salt("communication-choice")
        .width(210.0)
        .selected_text(format!("{}", communication_choice))
        .show_ui(ui, |ui| {
            spawn_choice(
                ui,
                communication_choice,
                CommunicationChoice::ReqRegistrationToChat,
            );
            spawn_choice(
                ui,
                communication_choice,
                CommunicationChoice::ReqClientsList,
            );
            spawn_choice(ui, communication_choice, CommunicationChoice::MessageSend);
        });
}

pub fn spawn_choice<V: PartialEq + Display + Copy>(
    ui: &mut Ui,
    current_value: &mut V,
    selected_value: V,
) {
    ui.selectable_value(current_value, selected_value, format!("{}", selected_value));
}

impl Display for MessageChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MessageChoice::NotChosen => "Choose request type",
            MessageChoice::ReqServerType => "Request server type",
            MessageChoice::Content => "Content request",
            MessageChoice::Communication => "Communication request",
        };
        write!(f, "{}", str)
    }
}

impl Display for ContentChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ContentChoice::NotChosen => "Choose content request",
            ContentChoice::ReqFilesList => "Request files list",
            ContentChoice::ReqFile => "Request file",
        };
        write!(f, "{}", str)
    }
}

impl Display for CommunicationChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            CommunicationChoice::NotChosen => "Choose communication request",
            CommunicationChoice::ReqRegistrationToChat => "Request registration to chat",
            CommunicationChoice::MessageSend => "Send message",
            CommunicationChoice::ReqClientsList => "Request clients list",
        };
        write!(f, "{}", str)
    }
}
