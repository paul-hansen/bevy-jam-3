use bevy::prelude::*;
use bevy_rapier2d::prelude::{
    Collider, ColliderMassProperties, Damping, ExternalForce, ExternalImpulse, GravityScale,
    MassProperties, RigidBody, Velocity,
};

pub mod lyon_rendering;

#[derive(Bundle)]
pub struct PhysicsBundle {
    collider: Collider,
    rb: RigidBody,
    velocity: Velocity,
    mass: ColliderMassProperties,
    damping: Damping,
    ex_force: ExternalForce,
    ex_impulse: ExternalImpulse,
    gravity: GravityScale,
}

impl Default for PhysicsBundle {
    fn default() -> Self {
        Self {
            collider: Collider::cuboid(16., 16.),
            rb: RigidBody::Dynamic,
            velocity: Default::default(),
            mass: ColliderMassProperties::MassProperties(MassProperties {
                local_center_of_mass: Vec2::new(16., 10.),
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
