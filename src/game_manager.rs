use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_replicon::renet::RenetServer;
use bevy_replicon::replication_core::Replication;
use egui::{Align2};
use rand::Rng;

use crate::forms::{ConnectForm, ListenForm};

use crate::{
    arena::{Arena, Force},
    asteroid::{asteroid_spawn, Asteroid},
    bundles::lyon_rendering::roid_paths::RoidPath,
};

#[derive(Debug, Hash, Eq, PartialEq, Clone, States, Default, Reflect)]
#[reflect(Default)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    PreGame,
    Playing,
    PostGame,
}

pub struct GameManager;

impl Plugin for GameManager {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>();
        app.add_systems((load_state,).in_schedule(OnEnter(GameState::Loading)));
        app.add_systems((draw_main_menu,).in_set(OnUpdate(GameState::MainMenu)));
        app.add_systems((build_level,).in_schedule(OnEnter(GameState::Playing)));

        app.add_systems((asteroid_spawn,));
    }
}

pub fn load_state(mut next_state: ResMut<NextState<GameState>>) {
    info!("Auto-Advancing To Main Menu State");
    next_state.set(GameState::MainMenu);
}

pub fn draw_main_menu(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut connect_form: Local<ConnectForm>,
    mut listen_form: Local<ListenForm>,
) {
    egui::Window::new("Main Menu")
        .auto_sized()
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Join Game");
            connect_form.draw(ui);
            if ui.button("Join").clicked() {
                if let Ok(connect) = connect_form.validate() {
                    commands.add(connect);
                }
            }
            ui.add_space(14.0);
            ui.push_id("host", |ui| {
                ui.heading("Host Game");
                listen_form.draw(ui);
                if ui.button("Host").clicked() {
                    if let Ok(listen) = listen_form.validate() {
                        commands.add(listen);
                    }
                }
            });
        });
}

///Should only be run by the server, and then fill backfill on the clients
pub fn build_level(mut cmds: Commands, server_resource: Option<Res<RenetServer>>, time: Res<Time>) {
    if server_resource.is_none() {
        return;
    }

    let arena_size = Vec2::new(600.0, 400.0);
    cmds.spawn(Arena {
        starting_size: arena_size,
        current_size: arena_size,
        time_spawned: time.elapsed_seconds(),
        friendly_force: Force::None,
    })
    .insert(Name::new("Arena"))
    .insert(Replication::default());

    let mut rng = rand::thread_rng();
    for _ in 0..5 {
        let roid_path = match (rng.gen_range(0..10) % 2) == 0 {
            true => RoidPath::One,
            false => RoidPath::Two,
        };

        let x = rng.gen_range(-300.0..300.0);
        let y = rng.gen_range(-300.0..300.0);
        let scale = rng.gen_range(16.0..64.0);
        let rotation = rng.gen_range(0.0..(PI * 2.0));

        let mut transform = Transform::from_xyz(x, y, 0.2);
        transform.rotate_z(rotation);
        cmds.spawn((
            Asteroid {
                scale,
                path: roid_path,
            },
            transform,
            Replication::default(),
        ));
    }
}
