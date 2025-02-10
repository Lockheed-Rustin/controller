use std::sync::{Arc, Mutex, MutexGuard};

use crossbeam_channel::{select_biased, Receiver};

use drone_networks::controller::ClientEvent;
use drone_networks::message::{ClientBody, ServerBody, ServerContentBody};
use eframe::egui::{Color32, ColorImage, TextureFilter, TextureOptions};
use wg_2024::network::NodeId;
use wg_2024::packet::{NodeType, Packet};
use crate::app::simulation_controller_ui::{ContentFile, ContentFileType};
use super::helper;
use crate::shared_data::SimulationData;

pub fn receiver_loop(
    data_ref: Arc<Mutex<SimulationData>>,
    rec_client: Receiver<ClientEvent>,
    rec_kill: Receiver<()>,
) {
    loop {
        select_biased! {
            recv(rec_kill) -> packet => {
                if packet.is_ok() {
                    return;
                }
            }
            recv(rec_client) -> packet => {
                if let Ok(event) = packet {
                    handle_event(&data_ref, event);
                }
            }
        }
    }
}

fn handle_event(data_ref: &Arc<Mutex<SimulationData>>, event: ClientEvent) {
    match event {
        ClientEvent::PacketSent(p) => handle_packet_sent(data_ref, &p),
        ClientEvent::PacketReceived(p, id) => handle_packet_received(data_ref, &p, id),
        ClientEvent::MessageAssembled { body, from, to } => {
            handle_message_assembled(data_ref, body, from, to);
        }
        ClientEvent::MessageFragmented { body, from, to } => {
            handle_message_fragmented(data_ref, body, from, to);
        }
    }
}

fn handle_packet_sent(data_ref: &Arc<Mutex<SimulationData>>, p: &Packet) {
    helper::handle_packet_sent(NodeType::Client, p, data_ref);
}

fn handle_packet_received(data_ref: &Arc<Mutex<SimulationData>>, p: &Packet, id: NodeId) {
    helper::handle_packet_received(id, NodeType::Client, p, data_ref);
}

fn handle_message_assembled(
    data_ref: &Arc<Mutex<SimulationData>>,
    body: ServerBody,
    from: NodeId,
    to: NodeId,
) {
    let mut log_line = format!("Assembled message from server #{from}\n");
    log_line.push_str(&helper::get_log_line_server_body(&body));
    let mut data = data_ref.lock().unwrap();
    data.add_log(to, log_line, Color32::WHITE);
    data.client_stats.get_mut(&to).unwrap().messages_assembled += 1;

    if let ServerBody::ServerContent(ServerContentBody::RespFile(v)) = body {
        load_file(&mut data, v);
    }
    data.ctx.request_repaint();
}

fn handle_message_fragmented(
    data_ref: &Arc<Mutex<SimulationData>>,
    body: ClientBody,
    from: NodeId,
    to: NodeId,
) {
    let mut log_line = format!("Fragmented message for server #{to}\n");
    log_line.push_str(&helper::get_log_line_client_body(&body));
    let mut data = data_ref.lock().unwrap();
    data.add_log(from, log_line, Color32::WHITE);
    data.client_stats
        .get_mut(&from)
        .unwrap()
        .messages_fragmented += 1;
    data.ctx.request_repaint();
}

fn load_file(
    data: &mut MutexGuard<SimulationData>,
    v: Vec<u8>
) {
    if infer::is_image(&v) {
        let image = image::load_from_memory(&v).expect("Failed to load image");
        let size = [image.width() as usize, image.height() as usize];
        let rgba = image.to_rgba8();
        let color_image = ColorImage::from_rgba_unmultiplied(size, &rgba);

        let opt = TextureOptions {
            magnification: TextureFilter::Nearest,
            minification: TextureFilter::Nearest,
            ..TextureOptions::default()
        };

        let texture = data.ctx.load_texture("my_texture", color_image, opt);
        data.files.push(ContentFile{
            name: "Immagine".to_string(),
            file: ContentFileType::Image(texture),
        });
    } else {
        let text = String::from_utf8_lossy(&v).to_string();
        data.files.push(ContentFile{
            name: "Testo".to_string(),
            file: ContentFileType::Text(text)
        });
    }
}
