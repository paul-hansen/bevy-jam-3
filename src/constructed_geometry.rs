use bevy::prelude::*;
use bevy_rapier2d::na::ComplexField;
use rand::Rng;

pub trait PointDistribution {
    fn point(&self) -> Vec2;
}

pub enum Shape {
    Circle(CircleShape, Vec2),
    Quad(QuadShape, Vec2),
    Arc(ArcSectionShape, Vec2),
}

pub struct ArcSectionShape {
    pub arc_length: f32, // Radians
    pub arc_angle: f32,  // Radians
    pub angle: f32,      // Radians
}

pub struct CircleShape {
    pub radius: f32,
    pub exclude_radius: Option<f32>,
}

impl PointDistribution for CircleShape {
    fn point(&self) -> Vec2 {
        let mut rng = rand::thread_rng();

        let theta = rng.gen_range(0. ..6.29);
        let (sin, cos) = theta.sin_cos();

        let mut e_radius = 0.;

        if let Some(exclude_radius) = self.exclude_radius {
            e_radius = exclude_radius;
        }

        let mut translation = rng.gen_range(0. ..1.);
        translation = ((self.radius - e_radius) * translation.sqrt()) + e_radius;

        Vec2 {
            x: cos * translation,
            y: sin * translation,
        }
    }
}
pub struct QuadShape {
    pub width: f32,
    pub height: f32,
}

impl PointDistribution for QuadShape {
    fn point(&self) -> Vec2 {
        let mut rng = rand::thread_rng();

        Vec2 {
            x: rng.gen_range(0. ..self.width),
            y: rng.gen_range(0. ..self.height),
        }
    }
}
