use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_prototype_lyon::prelude::{Fill, Path, PathBuilder, ShapeBundle, Stroke};
use bevy_prototype_lyon::render::ShapeMaterial;

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

/// Bundle that should be added to any clients (including the server if the server is playing)
/// Same as LyonRenderBundle but without the transform so we can set that when it is spawned.
#[derive(Bundle)]
pub struct LyonRenderBundleClient {
    pub path: Path,
    pub mesh: Mesh2dHandle,
    pub material: Handle<ShapeMaterial>,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub stroke: Stroke,
    pub fill: Fill,
}

impl Default for LyonRenderBundleClient {
    fn default() -> Self {
        Self {
            path: get_path_from_verts(&SHIP_PATH, Vec2::splat(32.)),
            mesh: Default::default(),
            material: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
            stroke: Stroke::new(Color::YELLOW, 3.0),
            fill: Fill::color(Color::rgba(0., 0., 0., 0.)),
        }
    }
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
        path_builder.line_to(Vec2::from(*point) * scale);
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
        (-0.17, -0.5),
        (-0.25, -0.3),
        (-0.5, -0.5),
        (0.0, 0.5),
        (0.5, -0.5),
        (0.25, -0.3),
        (0.16, -0.5),
    ];
}

/// A 1x1 square
pub const UNIT_SQUARE_PATH: [(f32, f32); 5] = [
    (-0.5, 0.5),
    (0.5, 0.5),
    (0.5, -0.5),
    (-0.5, -0.5),
    (-0.5, 0.5),
];

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
        (-0.4, -0.5),
        (-0.5, -0.4),
        (-0.3, 0.3),
        (0.16, 0.5),
        (0.5, 0.2),
        (0.3, -0.35),
        (0.0, -0.32),
        (-0.4, -0.5),
    ];
    pub const ROID_PATH2: [(f32, f32); 13] = [
        (-0.5, -0.1),
        (-0.3, 0.05),
        (-0.1, 0.5),
        (0.3, 0.5),
        (0.2, 0.16),
        (0.5, 0.05),
        (0.4, -0.35),
        (0.16, -0.58),
        (0.1, -0.5),
        (-0.2, -0.5),
        (-0.3, -0.3),
        (-0.4, -0.25),
        (-0.5, -0.1),
    ];
}

pub mod powerups {
    pub const RAPIDFIRE_PATH: [(f32, f32); 5] = [
        (-0.5, -0.5),
        (-0.5, 0.5),
        (0.5, -0.5),
        (0.5, 0.5),
        (-0.5, -0.5),
    ];

    pub const SCATTERGUN_PATH: [(f32, f32); 8] = [
        (-0.5, -0.5),
        (0.5, 0.5),
        (0.0, 0.0),
        (0.0, 0.5),
        (0.0, -0.5),
        (0.0, 0.0),
        (-0.5, 0.5),
        (0.5, -0.5),
    ];
}

pub mod ship_parts {
    pub const THRUSTER_JET: [(f32, f32); 7] = [
        (-0.5, -0.5 + 0.5),
        (-0.3, -0.3 + 0.5),
        (-0.1, -0.1 + 0.5),
        (0.0, 1.0 + 0.5),
        (0.1, -0.1 + 0.5),
        (0.3, -0.3 + 0.5),
        (0.5, -0.5 + 0.5),
    ];
}
