use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_replicon::renet::RenetServer;
use rand::Rng;

use crate::{
    asteroid::{asteroid_spawn, Asteroid},
    bundles::lyon_rendering::roid_paths::RoidPath,
};

#[derive(Debug, Hash, Eq, PartialEq, Clone, States, Default)]
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
        app.add_systems((main_menu_state,).in_schedule(OnEnter(GameState::MainMenu)));
        app.add_systems((build_level,).in_schedule(OnEnter(GameState::Playing)));

        app.add_systems((asteroid_spawn,));
    }
}

pub fn load_state(mut next_state: ResMut<NextState<GameState>>) {
    info!("Auto-Advancing To Main Menu State");
    next_state.set(GameState::MainMenu);
}

pub fn main_menu_state(mut next_state: ResMut<NextState<GameState>>) {
    info!("Auto-Advancing to PreGame State");
    next_state.set(GameState::PreGame);
}

///Should only be run by the server, and then fill backfill on the clients
pub fn build_level(mut cmds: Commands, server_resource: Option<Res<RenetServer>>) {
    if server_resource.is_none() {
        return;
    }

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
        ));
    }
}
