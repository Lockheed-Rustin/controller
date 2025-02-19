use drone_network::message::ClientBody::{ClientCommunication, ClientContent};
use drone_network::message::{
    ClientBody, ClientCommunicationBody, ClientContentBody, CommunicationMessage,
};
use eframe::egui::{vec2, Color32, ComboBox, Context, TextEdit, TextStyle, Ui, Window};
use std::fmt::{Display, Formatter};
use std::sync::MutexGuard;

use crate::app::simulation_controller_ui::ClientWindowState;
use crate::shared_data::SimulationData;
use crate::ui_components;
use wg_2024::network::NodeId;

/// Represents the choice of the message type in the "Actions" form section.
#[derive(PartialEq, Copy, Clone, Debug, Default)]
pub enum MessageChoice {
    #[default]
    NotChosen,
    ReqServerType,
    Content,
    Communication,
}

/// Represents the choice of the content message type in the "Actions" form section.
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum ContentChoice {
    #[default]
    NotChosen,
    ReqFilesList,
    ReqFile,
}

/// Represents the choice of the communication message type in the "Actions" form section.
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum CommunicationChoice {
    #[default]
    NotChosen,
    ReqRegistrationToChat,
    MessageSend,
    ReqClientsList,
}

/// Spawns the client window.
/// #Arguments
/// `id` is the id of the drone whose window needs to be spawned.
pub fn spawn(
    ctx: &Context,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    client_ids: &[NodeId],
    server_ids: &[NodeId],
    open: &mut bool,
    state: &mut ClientWindowState,
) {
    Window::new(format!("Client #{id}"))
        .open(open)
        .fixed_size(vec2(400.0, 300.0))
        .show(ctx, |ui| {
            ui_components::stats::spawn_client(ui, mutex, id);
            // logs
            ui_components::logs::spawn(ui, mutex, id);
            if ui.button("Clear log").clicked() {
                mutex.clear_log(id);
            }
            ui.add_space(5.0);

            // actions
            ui_components::text::spawn_white_heading(ui, "Actions");
            ui.add_space(5.0);

            spawn_message_combobox(ui, mutex, id, client_ids, server_ids, state);
        });
}

/// Spawns the form for choosing the message to be sent.
fn spawn_message_combobox(
    ui: &mut Ui,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    other_client_ids: &[NodeId],
    server_ids: &[NodeId],
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
            spawn_send_form(ui, mutex, id, other_client_ids, server_ids, state);
        }
        MessageChoice::Content => {
            state.communication_choice = CommunicationChoice::NotChosen;
            spawn_content_combobox(ui, mutex, id, other_client_ids, server_ids, state);
        }
        MessageChoice::Communication => {
            state.content_choice = ContentChoice::NotChosen;
            spawn_communication_combobox(ui, mutex, id, other_client_ids, server_ids, state);
        }
    }
}

/// Spawns the form for choosing the content message to be sent.
fn spawn_content_combobox(
    ui: &mut Ui,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    other_client_ids: &[NodeId],
    server_ids: &[NodeId],
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
        ContentChoice::ReqFilesList | ContentChoice::ReqFile => {
            spawn_send_form(ui, mutex, id, other_client_ids, server_ids, state);
        }
    };
}

/// Spawns the form for choosing the communication message to be sent.
fn spawn_communication_combobox(
    ui: &mut Ui,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    other_client_ids: &[NodeId],
    server_ids: &[NodeId],
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
        CommunicationChoice::ReqRegistrationToChat
        | CommunicationChoice::MessageSend
        | CommunicationChoice::ReqClientsList => {
            spawn_send_form(ui, mutex, id, other_client_ids, server_ids, state);
        }
    };
}

/// Spawns the form for sending the message.
fn spawn_send_form(
    ui: &mut Ui,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    other_client_ids: &[NodeId],
    server_ids: &[NodeId],
    state: &mut ClientWindowState,
) {
    ui.add_space(2.0);
    match state.message_choice {
        MessageChoice::Content => match state.content_choice {
            ContentChoice::NotChosen => {}
            ContentChoice::ReqFilesList => {
                ui.horizontal(|ui| {
                    spawn_server_combobox(ui, server_ids, state);
                    spawn_send_button(ui, mutex, id, state);
                });
            }
            ContentChoice::ReqFile => {
                ui.horizontal(|ui| {
                    spawn_server_combobox(ui, server_ids, state);
                    spawn_text_input_singleline(ui, state, 235.0);
                    spawn_send_button(ui, mutex, id, state);
                });
            }
        },
        MessageChoice::Communication => match state.communication_choice {
            CommunicationChoice::NotChosen => {}
            CommunicationChoice::ReqRegistrationToChat => {
                ui.horizontal(|ui| {
                    spawn_server_combobox(ui, server_ids, state);
                    spawn_send_button(ui, mutex, id, state);
                });
            }
            CommunicationChoice::ReqClientsList => {
                ui.horizontal(|ui| {
                    spawn_server_combobox(ui, server_ids, state);
                    spawn_send_button(ui, mutex, id, state);
                });
            }
            CommunicationChoice::MessageSend => {
                ui.horizontal(|ui| {
                    spawn_server_combobox(ui, server_ids, state);
                    spawn_client_combobox(ui, id, other_client_ids, state);
                });
                ui.horizontal(|ui| {
                    spawn_text_input_multiline(ui, state, 350.0);
                    spawn_send_button(ui, mutex, id, state);
                });
            }
        },
        MessageChoice::ReqServerType => {
            ui.horizontal(|ui| {
                spawn_server_combobox(ui, server_ids, state);
                spawn_send_button(ui, mutex, id, state);
            });
        }
        MessageChoice::NotChosen => {}
    }
}

/// Spawns the drop-down menu for choosing the server id.
fn spawn_server_combobox(ui: &mut Ui, server_ids: &[NodeId], state: &mut ClientWindowState) {
    ui.label("Server id:");
    let node_ids_copy_for_closure = server_ids.iter().copied();
    ComboBox::from_id_salt("server")
        .width(50.0)
        .selected_text(
            state
                .server_destination_id
                .map_or_else(|| "-".to_string(), |num| num.to_string()),
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

/// Spawns the drop-down menu for choosing the client id.
fn spawn_client_combobox(
    ui: &mut Ui,
    id: NodeId,
    client_ids: &[NodeId],
    state: &mut ClientWindowState,
) {
    ui.label("Destination (Client) id:");
    ComboBox::from_id_salt("client")
        .width(50.0)
        .selected_text(
            state
                .client_destination_id
                .map_or_else(|| "-".to_string(), |num| num.to_string()),
        )
        .show_ui(ui, |ui| {
            for number in client_ids.iter().filter(|i| **i != id) {
                ui.selectable_value(
                    &mut state.client_destination_id,
                    Some(*number),
                    number.to_string(),
                );
            }
        });
}

/// Spawns the multiline text input.
fn spawn_text_input_multiline(ui: &mut Ui, state: &mut ClientWindowState, width: f32) {
    ui.add(
        TextEdit::multiline(&mut state.text_input)
            .desired_width(width)
            .font(TextStyle::Monospace),
    );
}

/// Spawns the single line text input.
fn spawn_text_input_singleline(ui: &mut Ui, state: &mut ClientWindowState, width: f32) {
    ui.add(
        TextEdit::singleline(&mut state.text_input)
            .desired_width(width)
            .font(TextStyle::Monospace),
    );
}

/// Spawns the send button.
fn spawn_send_button(
    ui: &mut Ui,
    mutex: &mut MutexGuard<SimulationData>,
    id: NodeId,
    state: &mut ClientWindowState,
) {
    if ui.button("Send").clicked() {
        let log_line = match send(mutex, id, state) {
            None => "Error in sending command".to_string(),
            Some(()) => {
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

/// spawns a selectable item for drop-down menus.
fn spawn_choice<V: PartialEq + Display + Copy>(
    ui: &mut Ui,
    current_value: &mut V,
    selected_value: V,
) {
    ui.selectable_value(current_value, selected_value, format!("{selected_value}"));
}

/// Constructs the message with the form data and sends it.
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
        .ok()
}

impl Display for MessageChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MessageChoice::NotChosen => "Choose request type",
            MessageChoice::ReqServerType => "Request server type",
            MessageChoice::Content => "Content request",
            MessageChoice::Communication => "Communication request",
        };
        write!(f, "{str}")
    }
}

impl Display for ContentChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ContentChoice::NotChosen => "Choose content request",
            ContentChoice::ReqFilesList => "Request files list",
            ContentChoice::ReqFile => "Request file",
        };
        write!(f, "{str}")
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
        write!(f, "{str}")
    }
}
