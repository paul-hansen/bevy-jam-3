use crate::game_manager::GameState;
use crate::ui::MenuState;
use bevy::app::{App, Plugin};
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_editor_pls::editor_window::{EditorWindow, EditorWindowContext};
use bevy_editor_pls::egui::Ui;
use bevy_editor_pls::AddEditorWindow;
use bevy_inspector_egui::bevy_inspector::ui_for_state;
use bevy_replicon::prelude::*;
use renet_visualizer::{RenetClientVisualizer, RenetServerVisualizer, RenetVisualizerStyle};

pub struct EditorExtensionPlugin;

impl Plugin for EditorExtensionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RenetClientVisualizer::<200>::new(
            RenetVisualizerStyle::default(),
        ));
        app.insert_resource(RenetServerVisualizer::<200>::new(
            RenetVisualizerStyle::default(),
        ));
        app.add_editor_window::<RenetEditorWindow>();
        app.add_editor_window::<StatesEditorWindow>();
    }
}

pub struct RenetEditorWindow;

impl EditorWindow for RenetEditorWindow {
    type State = ();
    const NAME: &'static str = "Network";

    fn ui(world: &mut World, _: EditorWindowContext, ui: &mut Ui) {
        let mut state = SystemState::<(
            Option<ResMut<RenetClientVisualizer<200>>>,
            Option<Res<RenetClient>>,
            Option<ResMut<RenetServerVisualizer<200>>>,
            Option<Res<RenetServer>>,
        )>::new(world);
        let (mut client_visualizer, client, mut server_visualizer, server) = state.get_mut(world);
        if let (Some(client_visualizer), Some(client)) = (&mut client_visualizer, client) {
            client_visualizer.add_network_info(client.network_info());
            client_visualizer.draw_all(ui);
        } else if let (Some(server_visualizer), Some(server)) = (&mut server_visualizer, server) {
            server_visualizer.update(&server);
            server_ui(ui, server_visualizer, server.as_ref());
        } else {
            ui.label("No client or server added");
        }
    }
}

fn server_ui<const N: usize>(
    ui: &mut Ui,
    server_visualizer: &mut RenetServerVisualizer<N>,
    server: &RenetServer,
) {
    for client_id in server.clients_id().iter().cloned() {
        ui.vertical(|ui| {
            ui.heading(format!("Client {}", client_id));

            server_visualizer.draw_client_metrics(client_id, ui);
        });
    }
}

pub struct StatesEditorWindow;

impl EditorWindow for StatesEditorWindow {
    type State = ();
    const NAME: &'static str = "States";

    fn ui(world: &mut World, _cx: EditorWindowContext, ui: &mut Ui) {
        ui.push_id("game_state", |ui| {
            ui.label("Game State");
            ui_for_state::<GameState>(world, ui);
        });
        ui.push_id("menu_state", |ui| {
            ui.label("Menu State");
            ui_for_state::<MenuState>(world, ui);
        });
    }
}
