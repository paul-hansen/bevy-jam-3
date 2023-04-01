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

    //path_builder.move_to(path[0]);

    for point in path {
        path_builder.line_to(point * scale);
    }

    path_builder.build()
}

#[derive(Bundle)]
pub struct PlayerShipBundle {
    pub shape_render: ShapeBundle,
    pub stroke: Stroke,
    pub fill: Fill,
}

impl Default for PlayerShipBundle {
    fn default() -> Self {
        Self {
            shape_render: ShapeBundle {
                path: get_ship_path(32.),
                transform: Transform::from_xyz(0.0, 0.0, 0.5),
                ..default()
            },
            stroke: Stroke::new(Color::YELLOW, 5.0),
            fill: Fill::color(Color::rgba(0., 0., 0., 0.)),
        }
    }
}
