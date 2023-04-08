use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{Path, ShapeBundle, Stroke};
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties, MassProperties};
use serde::{Deserialize, Serialize};

use crate::bundles::{
    lyon_rendering::{
        get_path_from_verts,
        roid_paths::{RoidPath, ROID_PATH, ROID_PATH2},
        LyonRenderBundle,
    },
    PhysicsBundle,
};
use crate::health::Health;

#[derive(Component, Reflect, Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct Asteroid {
    pub scale: f32,
    pub path: RoidPath,
}

impl Default for Asteroid {
    fn default() -> Self {
        Self {
            scale: 16.0,
            path: RoidPath::One,
        }
    }
}

#[derive(Bundle)]
pub struct AsteroidBundle {
    physics: PhysicsBundle,
    render: LyonRenderBundle,
}

impl Default for AsteroidBundle {
    fn default() -> Self {
        Self {
            physics: PhysicsBundle {
                mass: ColliderMassProperties::MassProperties(MassProperties {
                    mass: 50.0,
                    ..default()
                }),
                ..Default::default()
            },
            render: LyonRenderBundle {
                shape_render: ShapeBundle {
                    path: get_path_from_verts(&ROID_PATH, Vec2::splat(48.0)),
                    ..default()
                },
                stroke: Stroke::new(Color::ALICE_BLUE, 2.0),
                ..Default::default()
            },
        }
    }
}

impl From<(Transform, Path)> for AsteroidBundle {
    fn from(value: (Transform, Path)) -> Self {
        let (transform, path) = value;
        AsteroidBundle {
            render: LyonRenderBundle {
                shape_render: ShapeBundle {
                    path,
                    transform,
                    ..Default::default()
                },
                ..default()
            },
            ..Default::default()
        }
    }
}

//TODO: Consider how to group functions like this
pub fn asteroid_spawn(
    new_roids: Query<(Entity, &Asteroid, &Transform), Added<Asteroid>>,
    mut cmds: Commands,
) {
    new_roids.for_each(|(ent, asteroid, transform)| {
        let roid_path = match asteroid.path {
            RoidPath::One => ROID_PATH.to_vec(),
            RoidPath::Two => ROID_PATH2.to_vec(),
        };

        cmds.entity(ent).insert((
            AsteroidBundle {
                physics: PhysicsBundle {
                    collider: Collider::cuboid(asteroid.scale * 0.5, asteroid.scale * 0.5),
                    mass: ColliderMassProperties::Mass(5.0),
                    ..Default::default()
                },
                ..AsteroidBundle::from((
                    *transform,
                    get_path_from_verts(&roid_path, Vec2::splat(asteroid.scale)),
                ))
            },
            Health{
                current: 200.0,
                max: 200.0
            },
            Name::new("Asteroid"),
        ));
    });
}
