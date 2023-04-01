use bevy::{prelude::*};
use bevy_prototype_lyon::prelude::{ShapeBundle, Stroke};

use crate::bundles::player_ship::get_ship_path;

use self::player_ship::PlayerShipBundle;

pub mod player_ship;

pub struct TestRenderingPlugin;

impl Plugin for TestRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_ship_system);
    }
}

pub fn spawn_ship_system(mut commands: Commands){
    info!("Spawning Ship");

    commands.spawn(PlayerShipBundle{
        shape_render: ShapeBundle{
            path: get_ship_path(32.0),
            transform: Transform::from_xyz(50.0, 50.0, 0.5),
            ..default()
        },
        stroke: Stroke::new(Color::YELLOW, 2.0),
        ..default()
    }).insert(Name::new("Player"));
}