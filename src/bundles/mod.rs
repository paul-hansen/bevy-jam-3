use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{ShapeBundle, Stroke, PathBuilder, Path};

use self::player_ship::PlayerShipBundle;

pub mod player_ship;

pub struct TestRenderingPlugin;

impl Plugin for TestRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_ship_system);
    }
}

pub fn get_path_from_verts(points: Vec<Vec2>, scale: f32) -> Path{
    let mut path_builder = PathBuilder::new();

    for point in points {
        path_builder.line_to(point * scale);
    }

    path_builder.build()
}

pub fn spawn_ship_system(mut commands: Commands) {
    info!("Spawning Ship");

    commands
        .spawn(PlayerShipBundle::default())
        .insert(Name::new("Player"));
}
