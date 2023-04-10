use crate::network::commands::Connect;
use crate::network::DEFAULT_PORT;
use bevy::prelude::*;
use bevy_egui::EguiContexts;
use egui::{Align2, Color32, Ui, Widget};
use std::net::{AddrParseError, IpAddr};
use std::str::FromStr;

pub fn draw_join_by_ip(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut connect_form: Local<ConnectForm>,
) {
    egui::Window::new("Join by IP address")
        .auto_sized()
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .show(contexts.ctx_mut(), |ui| {
            connect_form.draw(ui);
            if ui.button("Join").clicked() {
                if let Ok(connect) = connect_form.validate() {
                    commands.add(connect);
                }
            }
        });
}

pub struct ConnectForm {
    pub ip: String,
    pub port: u16,
    pub bind: String,
    pub error: Option<String>,
}

impl ConnectForm {
    pub fn validate(&mut self) -> Result<Connect, AddrParseError> {
        Ok(Connect {
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

impl Default for ConnectForm {
    fn default() -> Self {
        Self {
            ip: Connect::default().ip.to_string(),
            port: DEFAULT_PORT,
            bind: Connect::default().bind.to_string(),
            error: None,
        }
    }
}
