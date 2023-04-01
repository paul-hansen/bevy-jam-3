use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{Fill, Path, PathBuilder, ShapeBundle, Stroke};

pub fn get_ship_path(scale: f32) -> Path {
    let mut path_builder = PathBuilder::new();

    let path = vec![
        Vec2::new(0.33, 0.0),
        Vec2::new(0.25, 0.2),
        Vec2::new(0.0, 0.0),
        Vec2::new(0.5, 1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.75, 0.2),
        Vec2::new(0.66, 0.0),
    ];

    path.iter().for_each(|vertex| {
        path_builder.move_to(*vertex * scale);
    });

    path_builder.build()
}

#[derive(Bundle)]
pub struct PlayerShipBundle {
    shape_render: ShapeBundle,
    stroke: Stroke,
    fill: Fill,
}

impl Default for PlayerShipBundle {
    fn default() -> Self {
        Self {
            shape_render: ShapeBundle {
                path: get_ship_path(32.),
                ..default()
            },
            stroke: Stroke::new(Color::YELLOW, 2.0),
            fill: Fill::color(Color::rgba(0.0, 0.0, 0.0, 0.0)),
        }
    }
}
