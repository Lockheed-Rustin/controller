use drone_networks::message::ClientBody::{ClientCommunication, ClientContent};
use drone_networks::message::{
    ClientBody, ClientCommunicationBody, ClientContentBody, CommunicationMessage,
};
use eframe::egui::{vec2, Color32, ComboBox, Context, TextEdit, TextStyle, Ui, Window};
use std::fmt::{Display, Formatter};
use std::sync::MutexGuard;

use crate::app::ClientWindowState;
use crate::data::SimulationData;
use crate::ui_components;
use wg_2024::network::NodeId;

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
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    node_ids: Vec<NodeId>,
    open: &mut bool,
    state: &mut ClientWindowState,
) {
    Window::new(format!("Client #{}", id))
        .open(open)
        .fixed_size(vec2(400.0, 300.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("fragment").clicked() {
                    mutex.sc.send_fragment_fair(id);
                }
                if ui.button("flood").clicked() {
                    mutex.sc.send_flood_request_fair(id);
                }
                if ui.button("ack").clicked() {
                    mutex.sc.send_ack_fair(id);
                }
            });
            ui_components::stats::spawn_client_stats(ui, mutex, id);
            // logs
            ui_components::logs::spawn_logs(ui, mutex, id);
            if ui.button("Clear log").clicked() {
                mutex.clear_log(id);
            }
            ui.add_space(5.0);

            // actions
            ui_components::text::spawn_white_heading(ui, "Actions");
            ui.add_space(5.0);

            spawn_message_combobox(ui, mutex, id, node_ids, state);
        });
}

fn spawn_message_combobox(
    ui: &mut Ui,
    mutex: &mut MutexGuard<SimulationData>,
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
        MessageChoice::ReqServerType => {
            state.communication_choice = CommunicationChoice::NotChosen;
            state.content_choice = ContentChoice::NotChosen;
            spawn_send_form(ui, mutex, id, node_ids, state);
        }
        MessageChoice::Content => {
            state.communication_choice = CommunicationChoice::NotChosen;
            spawn_content_combobox(ui, mutex, id, node_ids, state);
        }
        MessageChoice::Communication => {
            state.content_choice = ContentChoice::NotChosen;
            spawn_communication_combobox(ui, mutex, id, node_ids, state);
        }
    }
}

fn spawn_content_combobox(
    ui: &mut Ui,
    mutex: &mut MutexGuard<SimulationData>,
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
        ContentChoice::ReqFilesList => spawn_send_form(ui, mutex, id, node_ids, state),
        ContentChoice::ReqFile => spawn_send_form(ui, mutex, id, node_ids, state),
    }
}

fn spawn_communication_combobox(
    ui: &mut Ui,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    node_ids: Vec<NodeId>,
    state: &mut ClientWindowState,
) {
    ComboBox::from_id_salt("communication-choice")
        .width(210.0)
        .selected_text(format!("{}", state.communication_choice))
        .show_ui(ui, |ui| {
            spawn_choice(
                ui,
                &mut state.communication_choice,
                CommunicationChoice::ReqRegistrationToChat,
            );
            spawn_choice(
                ui,
                &mut state.communication_choice,
                CommunicationChoice::ReqClientsList,
            );
            spawn_choice(
                ui,
                &mut state.communication_choice,
                CommunicationChoice::MessageSend,
            );
        });
    match state.communication_choice {
        CommunicationChoice::NotChosen => {}
        CommunicationChoice::ReqRegistrationToChat => {
            spawn_send_form(ui, mutex, id, node_ids, state)
        }
        CommunicationChoice::MessageSend => spawn_send_form(ui, mutex, id, node_ids, state),
        CommunicationChoice::ReqClientsList => spawn_send_form(ui, mutex, id, node_ids, state),
    }
}

fn spawn_send_form(
    ui: &mut Ui,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    node_ids: Vec<NodeId>,
    state: &mut ClientWindowState,
) {
    ui.add_space(2.0);
    match state.message_choice {
        MessageChoice::Content => match state.content_choice {
            ContentChoice::NotChosen => {}
            ContentChoice::ReqFilesList => {
                ui.horizontal(|ui| {
                    spawn_server_combobox(ui, node_ids.clone(), state);
                    spawn_send_button(ui, mutex, id, state);
                });
            }
            ContentChoice::ReqFile => {
                ui.horizontal(|ui| {
                    spawn_server_combobox(ui, node_ids.clone(), state);
                    spawn_text_input(ui, state, 235.0);
                    spawn_send_button(ui, mutex, id, state);
                });
            }
        },
        MessageChoice::Communication => match state.communication_choice {
            CommunicationChoice::NotChosen => {}
            CommunicationChoice::ReqRegistrationToChat => {
                ui.horizontal(|ui| {
                    spawn_server_combobox(ui, node_ids.clone(), state);
                    spawn_send_button(ui, mutex, id, state);
                });
            }
            CommunicationChoice::ReqClientsList => {
                ui.horizontal(|ui| {
                    spawn_server_combobox(ui, node_ids.clone(), state);
                    spawn_send_button(ui, mutex, id, state);
                });
            }
            CommunicationChoice::MessageSend => {
                ui.horizontal(|ui| {
                    spawn_server_combobox(ui, node_ids.clone(), state);
                    spawn_client_combobox(ui, node_ids.clone(), state);
                });
                ui.horizontal(|ui| {
                    spawn_text_input(ui, state, 310.0);
                    spawn_send_button(ui, mutex, id, state);
                });
            }
        },
        MessageChoice::ReqServerType => {
            ui.horizontal(|ui| {
                spawn_server_combobox(ui, node_ids.clone(), state);
                spawn_send_button(ui, mutex, id, state);
            });
        }
        _ => {}
    }
}

fn send(
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    state: &mut ClientWindowState,
) -> Option<()> {
    let client_body = match state.message_choice {
        MessageChoice::NotChosen => return None,
        MessageChoice::ReqServerType => ClientBody::ReqServerType,
        MessageChoice::Content => {
            let client_content_body = match state.content_choice {
                ContentChoice::NotChosen => return None,
                ContentChoice::ReqFilesList => ClientContentBody::ReqFilesList,
                ContentChoice::ReqFile => ClientContentBody::ReqFile(state.text_input.clone()),
            };
            ClientContent(client_content_body)
        }
        MessageChoice::Communication => {
            let client_communication_body = match state.communication_choice {
                CommunicationChoice::NotChosen => return None,
                CommunicationChoice::ReqRegistrationToChat => {
                    ClientCommunicationBody::ReqRegistrationToChat
                }
                CommunicationChoice::ReqClientsList => ClientCommunicationBody::ReqClientList,
                CommunicationChoice::MessageSend => {
                    let communication_message = CommunicationMessage {
                        from: id,
                        to: state.client_destination_id?,
                        message: state.text_input.clone(),
                    };
                    ClientCommunicationBody::MessageSend(communication_message)
                }
            };
            ClientCommunication(client_communication_body)
        }
    };
    mutex
        .sc
        .client_send_message(id, state.server_destination_id?, client_body)
}

fn spawn_choice<V: PartialEq + Display + Copy>(
    ui: &mut Ui,
    current_value: &mut V,
    selected_value: V,
) {
    ui.selectable_value(current_value, selected_value, format!("{}", selected_value));
}

fn spawn_server_combobox(ui: &mut Ui, node_ids: Vec<NodeId>, state: &mut ClientWindowState) {
    ui.label("Server id:");
    let node_ids_copy_for_closure = node_ids.iter().copied();
    ComboBox::from_id_salt("server")
        .width(50.0)
        .selected_text(
            state
                .server_destination_id
                .map(|num| num.to_string())
                .unwrap_or_else(|| "-".to_string()),
        )
        .show_ui(ui, |ui| {
            for number in node_ids_copy_for_closure {
                ui.selectable_value(
                    &mut state.server_destination_id,
                    Some(number),
                    number.to_string(),
                );
            }
        });
}

fn spawn_client_combobox(ui: &mut Ui, node_ids: Vec<NodeId>, state: &mut ClientWindowState) {
    ui.label("Destination (Client) id:");
    ComboBox::from_id_salt("client")
        .width(50.0)
        .selected_text(
            state
                .client_destination_id
                .map(|num| num.to_string())
                .unwrap_or_else(|| "-".to_string()),
        )
        .show_ui(ui, |ui| {
            for number in node_ids {
                ui.selectable_value(
                    &mut state.client_destination_id,
                    Some(number),
                    number.to_string(),
                );
            }
        });
}

fn spawn_text_input(ui: &mut Ui, state: &mut ClientWindowState, width: f32) {
    ui.add(
        TextEdit::multiline(&mut state.text_input)
            .desired_width(width)
            .font(TextStyle::Monospace),
    );
}

fn spawn_send_button(
    ui: &mut Ui,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    state: &mut ClientWindowState,
) {
    if ui.button("Send").clicked() {
        let log_line = match send(mutex, id, state) {
            None => "Error in sending command".to_string(),
            Some(_) => {
                state.message_choice = MessageChoice::NotChosen;
                state.content_choice = ContentChoice::NotChosen;
                state.communication_choice = CommunicationChoice::NotChosen;
                state.server_destination_id = None;
                state.client_destination_id = None;
                state.text_input.clear();
                "ClientCommand sent correctly".to_string()
            }
        };
        mutex.add_log(id, log_line, Color32::GRAY);
    }
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
