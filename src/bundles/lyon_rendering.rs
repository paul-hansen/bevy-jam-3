use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{Fill, Path, PathBuilder, ShapeBundle, Stroke};

use self::ship_paths::SHIP_PATH;

pub struct TestRenderingPlugin;

impl Plugin for TestRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_test_renders);
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
                path: get_path_from_verts(&SHIP_PATH, Vec2::splat(32.)),
                transform: Transform::from_xyz(0.0, 0.0, 0.5),
                ..default()
            },
            stroke: Stroke::new(Color::YELLOW, 3.0),
            fill: Fill::color(Color::rgba(0., 0., 0., 0.)),
        }
    }
}

pub fn get_path_from_verts(points: &[(f32, f32)], scale: Vec2) -> Path {
    let mut path_builder = PathBuilder::new();

    for point in points {
        //Subtract 0.5 to center
        let pt = Vec2::from(*point) - 0.5;
        path_builder.line_to(pt * scale);
    }

    path_builder.build()
}

pub fn spawn_test_renders(mut commands: Commands) {
    commands.spawn(LyonRenderBundle {
        shape_render: ShapeBundle {
            path: get_path_from_verts(&roid_paths::ROID_PATH, Vec2::splat(48.0)),
            transform: Transform::from_xyz(0.0, 200.0, 0.1),
            ..default()
        },
        ..default()
    });

    commands.spawn(LyonRenderBundle {
        shape_render: ShapeBundle {
            path: get_path_from_verts(&roid_paths::ROID_PATH2, Vec2::splat(48.0)),
            transform: Transform::from_xyz(150.0, 200.0, 0.1),
            ..default()
        },
        ..default()
    });
}

pub mod ship_paths {
    pub const SHIP_PATH: [(f32, f32); 7] = [
        (0.33, 0.0),
        (0.25, 0.2),
        (0.0, 0.0),
        (0.5, 1.0),
        (1.0, 0.0),
        (0.75, 0.2),
        (0.66, 0.0),
    ];
}

pub mod projectile_paths {
    pub const LASER_PATH: [(f32, f32); 2] = [(0.0, 0.0), (0.0, 10.0)];
}

pub mod roid_paths {
    use bevy::reflect::Reflect;
    use serde::{Deserialize, Serialize};

    #[derive(Copy, Clone, Reflect, Serialize, Deserialize, Eq, Debug, PartialEq)]
    pub enum RoidPath {
        One,
        Two,
    }

    pub const ROID_PATH: [(f32, f32); 8] = [
        (0.1, 0.0),
        (0.0, 0.1),
        (0.2, 0.8),
        (0.66, 1.0),
        (1.0, 0.7),
        (0.8, 0.15),
        (0.5, 0.12),
        (0.1, 0.0),
    ];
    pub const ROID_PATH2: [(f32, f32); 13] = [
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
        (0.0, 0.4),
    ];
}
