use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{Path, ShapeBundle, Stroke};
use bevy_replicon::replication_core::Replication;

use crate::bundles::{
    lyon_rendering::{get_path_from_verts, roid_paths::ROID_PATH, LyonRenderBundle},
    PhysicsBundle,
};

#[derive(Component, Default)]
pub struct Asteroid;

#[derive(Bundle)]
pub struct AsteroidBundle {
    physics: PhysicsBundle,
    render: LyonRenderBundle,
    replication: Replication,
    asteroid: Asteroid,
}

impl Default for AsteroidBundle {
    fn default() -> Self {
        Self {
            physics: Default::default(),
            render: LyonRenderBundle {
                shape_render: ShapeBundle {
                    path: get_path_from_verts(ROID_PATH.to_vec(), 48.0),
                    ..default()
                },
                stroke: Stroke::new(Color::ALICE_BLUE, 2.0),
                ..Default::default()
            },
            replication: Default::default(),
            asteroid: Default::default(),
        }
    }
}

pub fn spawn_roid(transform: Transform, path: Path, mut cmds: Commands) {
    cmds.spawn(AsteroidBundle {
        render: LyonRenderBundle {
            shape_render: ShapeBundle {
                path,
                transform,
                ..Default::default()
            },
            ..default()
        },
        ..Default::default()
    });
}
