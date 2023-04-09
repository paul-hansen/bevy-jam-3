use crate::network::commands::Listen;
use crate::network::{NetworkInfo, DEFAULT_PORT};
use bevy::prelude::*;
use bevy_egui::EguiContexts;
use egui::{Align2, Color32, Ui, Widget};
use std::net::{AddrParseError, IpAddr, Ipv4Addr};
use std::str::FromStr;

pub fn draw_create_game(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut listen_form: Local<ListenForm>,
    network_info: Res<NetworkInfo>,
) {
    if network_info.is_changed() {
        if let Some(ip) = network_info.public_ip {
            listen_form.ip = ip.to_string();
        }
    }
    egui::Window::new("Create Game")
        .auto_sized()
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .show(contexts.ctx_mut(), |ui| {
            listen_form.draw(ui);
            if ui.button("Host").clicked() {
                if let Ok(listen) = listen_form.validate() {
                    commands.add(listen);
                }
            }
        });
}

pub struct ListenForm {
    pub ip: String,
    pub port: u16,
    pub bind: String,
    pub error: Option<String>,
}

impl ListenForm {
    pub fn validate(&mut self) -> Result<Listen, AddrParseError> {
        Ok(Listen {
            bind: IpAddr::from_str(self.bind.as_str()).map_err(|e| {
                self.error = Some(e.to_string());
                e
            })?,
            ip: IpAddr::from_str(self.ip.as_str()).map_err(|e| {
                self.error = Some(e.to_string());
                e
            })?,
            port: self.port,
        })
    }

    pub fn draw(&mut self, ui: &mut Ui) {
        ui.label("IP Address");
        if ui.text_edit_singleline(&mut self.ip).changed() {
            self.error = None;
        }
        ui.collapsing("Advanced", |ui| {
            ui.label("Bind IP Address");
            if ui.text_edit_singleline(&mut self.bind).changed() {
                self.error = None;
            }
            ui.label("Port");
            if egui::DragValue::new(&mut self.port).ui(ui).changed() {
                self.error = None;
            }
        });

        if let Some(error_message) = &self.error {
            ui.colored_label(Color32::RED, error_message);
        }
    }
}

impl FromWorld for ListenForm {
    fn from_world(world: &mut World) -> Self {
        Self {
            ip: world
                .get_resource::<NetworkInfo>()
                .and_then(|i| i.public_ip)
                .unwrap_or(Ipv4Addr::new(127, 0, 0, 1).into())
                .to_string(),
            port: DEFAULT_PORT,
            bind: Ipv4Addr::new(0, 0, 0, 0).to_string(),
            error: None,
        }
    }
}
