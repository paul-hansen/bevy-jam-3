use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{Fill, ShapeBundle, Stroke};
use lazy_static::lazy_static;

use super::get_path_from_verts;

lazy_static!{
    pub static ref SHIP_PATH: Vec<Vec2> =  vec![
        Vec2::new(0.33, 0.0),
        Vec2::new(0.25, 0.2),
        Vec2::new(0.0, 0.0),
        Vec2::new(0.5, 1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.75, 0.2),
        Vec2::new(0.66, 0.0),
    ];
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
                path: get_path_from_verts(SHIP_PATH.to_vec(), 32.),
                transform: Transform::from_xyz(0.0, 0.0, 0.5),
                ..default()
            },
            stroke: Stroke::new(Color::YELLOW, 3.0),
            fill: Fill::color(Color::rgba(0., 0., 0., 0.)),
        }
    }
}
