use std::fmt::{Display, Formatter};
use std::sync::MutexGuard;

use eframe::egui::{vec2, ComboBox, Context, Key, TextEdit, TextStyle, Ui, Window};

use wg_2024::network::NodeId;
use crate::app::ClientWindowState;
use crate::data::SimulationData;
use crate::ui_components;

#[derive(PartialEq, Copy, Clone, Debug, Default)]
pub enum MessageChoice {
    #[default]
    NotChosen,
    ReqServerType,
    Content,
    Communication,
}

#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum ContentChoice {
    #[default]
    NotChosen,
    ReqFilesList,
    ReqFile,
}

#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum CommunicationChoice {
    #[default]
    NotChosen,
    ReqRegistrationToChat,
    MessageSend,
    ReqClientsList,
}

pub fn spawn_client_window(
    ctx: &Context,
    mut mutex: MutexGuard<SimulationData>,
    id: NodeId,
    node_ids: Vec<NodeId>,
    open: &mut bool,
    state: &mut ClientWindowState,
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

            spawn_message_combobox(ui, ctx, mutex, id, node_ids, state);

        });
}

fn spawn_message_combobox(
    ui: &mut Ui,
    ctx: &Context,
    mut mutex: MutexGuard<SimulationData>,
    id: NodeId,
    node_ids: Vec<NodeId>,
    state: &mut ClientWindowState,
) {
    ComboBox::from_id_salt("message-choice")
        .width(210.0)
        .selected_text(format!("{}", state.message_choice))
        .show_ui(ui, |ui| {
            spawn_choice(ui, &mut state.message_choice, MessageChoice::ReqServerType);
            spawn_choice(ui, &mut state.message_choice, MessageChoice::Content);
            spawn_choice(ui, &mut state.message_choice, MessageChoice::Communication);
        });
    match state.message_choice {
        MessageChoice::NotChosen => {}
        MessageChoice::ReqServerType => {}
        MessageChoice::Content => {
            state.communication_choice = CommunicationChoice::NotChosen;
            spawn_content_combobox(ui, ctx, mutex, id, node_ids, state);
        }
        MessageChoice::Communication => {
            state.content_choice = ContentChoice::NotChosen;
            spawn_communication_combobox(ui, ctx, mutex, id, node_ids, state);
        }
    }
}

fn spawn_content_combobox(
    ui: &mut Ui,
    ctx: &Context,
    mut mutex: MutexGuard<SimulationData>,
    id: NodeId,
    node_ids: Vec<NodeId>,
    state: &mut ClientWindowState,
) {
    ComboBox::from_id_salt("content-choice")
        .width(210.0)
        .selected_text(format!("{}", state.content_choice))
        .show_ui(ui, |ui| {
            spawn_choice(ui, &mut state.content_choice, ContentChoice::ReqFile);
            spawn_choice(ui, &mut state.content_choice, ContentChoice::ReqFilesList);
        });
    match state.content_choice {
        ContentChoice::NotChosen => {}
        ContentChoice::ReqFilesList => {
            spawn_send_form(ui, ctx, mutex, id, node_ids, state, false)
        }
        ContentChoice::ReqFile => {
            spawn_send_form(ui, ctx, mutex, id, node_ids, state, true)
        }
    }
}

fn spawn_communication_combobox(
    ui: &mut Ui,
    ctx: &Context,
    mut mutex: MutexGuard<SimulationData>,
    id: NodeId,
    node_ids: Vec<NodeId>,
    state: &mut ClientWindowState,
) {
    ComboBox::from_id_salt("communication-choice")
        .width(210.0)
        .selected_text(format!("{}", state.communication_choice))
        .show_ui(ui, |ui| {
            spawn_choice(ui, &mut state.communication_choice, CommunicationChoice::ReqRegistrationToChat);
            spawn_choice(ui, &mut state.communication_choice, CommunicationChoice::ReqClientsList);
            spawn_choice(ui, &mut state.communication_choice, CommunicationChoice::MessageSend);
        });
    match state.communication_choice {
        CommunicationChoice::NotChosen => {}
        CommunicationChoice::ReqRegistrationToChat => {
            spawn_send_form(ui, ctx, mutex, id, node_ids, state, false)
        }
        CommunicationChoice::MessageSend => {
            spawn_send_form(ui, ctx, mutex, id, node_ids, state, true)
        }
        CommunicationChoice::ReqClientsList => {
            spawn_send_form(ui, ctx, mutex, id, node_ids, state, false)
        }
    }
}

fn spawn_send_form(
    ui: &mut Ui,
    ctx: &Context,
    mut mutex: MutexGuard<SimulationData>,
    id: NodeId,
    node_ids: Vec<NodeId>,
    state: &mut ClientWindowState,
    with_text_input: bool,
) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.label("Destination:");
        ComboBox::from_id_salt("destination")
            .width(50.0)
            .selected_text(
                state.destination_id
                    .map(|num| num.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            )
            .show_ui(ui, |ui| {
                for number in node_ids {
                    ui.selectable_value(&mut state.destination_id, Some(number), number.to_string());
                }
            });
        if with_text_input {
            ui.add(
                TextEdit::singleline(&mut state.text_input)
                    .desired_width(210.0)
                    .font(TextStyle::Monospace),
            );
        }
        // if command_line_response.lost_focus()
        //     && ui.input(|i| i.key_pressed(Key::Enter))
        // {
        //     //log.push_str(format!("\n{}", line).as_str());
        //     line.clear();
        //     command_line_response.request_focus();
        // }
        if ui.button("Send").clicked() {
            println!("send");
        }
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
