use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{Fill, Path, PathBuilder, ShapeBundle, Stroke};

use crate::bundles::roid_paths::{ROID_PATH, ROID_PATH2};

use self::ship_paths::SHIP_PATH;

pub mod asteroid_bundle;
pub mod player_ship;

pub struct TestRenderingPlugin;

impl Plugin for TestRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_ship_system);
    }
}

#[derive(Bundle)]
pub struct LyonRenderBundle {
    pub shape_render: ShapeBundle,
    pub stroke: Stroke,
    pub fill: Fill,
}

impl Default for LyonRenderBundle {
    fn default() -> Self {
        Self {
            shape_render: ShapeBundle {
                path: get_path_from_verts(SHIP_PATH.to_vec(), 32.),
                transform: Transform::from_xyz(0.0, 0.0, 0.5),
                ..default()
            },
            stroke: Stroke::new(Color::YELLOW, 3.0),
            fill: Fill::color(Color::rgba(0., 0., 0., 0.)),
        }
    }
}

pub fn get_path_from_verts(points: Vec<(f32, f32)>, scale: f32) -> Path {
    let mut path_builder = PathBuilder::new();

    for point in points {
        path_builder.line_to(Vec2::from(point) * scale);
    }

    path_builder.build()
}

pub fn spawn_ship_system(mut commands: Commands) {
    info!("Spawning Ship");

    commands
        .spawn(LyonRenderBundle::default())
        .insert(Name::new("Player"));

    commands.spawn(LyonRenderBundle {
        shape_render: ShapeBundle {
            path: get_path_from_verts(ROID_PATH.to_vec(), 48.0),
            transform: Transform::from_xyz(0.0, 200.0, 0.1),
            ..default()
        },
        ..default()
    });

    commands.spawn(LyonRenderBundle {
        shape_render: ShapeBundle {
            path: get_path_from_verts(ROID_PATH2.to_vec(), 48.0),
            transform: Transform::from_xyz(150.0, 200.0, 0.1),
            ..default()
        },
        ..default()
    });
}

mod ship_paths {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref SHIP_PATH: Vec<(f32, f32)> = vec![
            (0.33, 0.0),
            (0.25, 0.2),
            (0.0, 0.0),
            (0.5, 1.0),
            (1.0, 0.0),
            (0.75, 0.2),
            (0.66, 0.0),
        ];
    }
}

mod roid_paths {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref ROID_PATH: Vec<(f32, f32)> = vec![
            (0.1, 0.0),
            (0.0, 0.1),
            (0.2, 0.8),
            (0.66, 1.0),
            (1.0, 0.7),
            (0.8, 0.15),
            (0.5, 0.12),
            (0.1, 0.0),
        ];
        pub static ref ROID_PATH2: Vec<(f32, f32)> = vec![
            (0.0, 0.4),
            (0.2, 0.55),
            (0.4, 1.0),
            (0.8, 1.0),
            (0.7, 0.66),
            (1.0, 0.55),
            (0.9, 0.15),
            (0.66, 0.08),
            (0.6, 0.0),
            (0.3, 0.0),
            (0.2, 0.2),
            (0.1, 0.25),
            (0.0, 0.4)
        ];
    }
}
