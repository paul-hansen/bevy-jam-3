use bevy::prelude::*;
use bevy_rapier2d::prelude::{
    Collider, ColliderMassProperties, Damping, ExternalForce, ExternalImpulse, GravityScale,
    MassProperties, RigidBody, Velocity,
};

pub mod lyon_rendering;

#[derive(Bundle)]
pub struct PhysicsBundle {
    pub collider: Collider,
    pub rb: RigidBody,
    pub velocity: Velocity,
    pub mass: ColliderMassProperties,
    pub damping: Damping,
    pub ex_force: ExternalForce,
    pub ex_impulse: ExternalImpulse,
    pub gravity: GravityScale,
}

impl Default for PhysicsBundle {
    fn default() -> Self {
        Self {
            collider: Collider::cuboid(16., 16.),
            rb: RigidBody::Dynamic,
            velocity: Default::default(),
            mass: ColliderMassProperties::MassProperties(MassProperties {
                local_center_of_mass: Vec2::new(0., 0.),
                mass: 1.0,
                ..default()
            }),
            damping: Damping {
                linear_damping: 0.2,
                angular_damping: 0.2,
            },
            ex_force: Default::default(),
            ex_impulse: Default::default(),
            gravity: GravityScale(0.0),
        }
    }
}
