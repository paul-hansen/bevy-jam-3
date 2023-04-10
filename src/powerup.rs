use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, QueryFilter, RapierContext, Sensor};
use bevy_replicon::{server::ServerSet, replication_core::AppReplicationExt};

use crate::{
    bundles::{lyon_rendering::LyonRenderBundle, PhysicsBundle},
    player::Player,
};

#[derive(Component, Reflect, Debug, Clone)]
pub enum PowerUp {
    HitherThither,
    TripleShot,
    RapidFire(u32),
    Shield(u32),
}

#[derive(Component, Reflect, Debug, Clone)]
pub enum Debuff {
    Slowed,
    Inaccuracy,
    HealthBurn,
    ReversedControls,
}

#[derive(Component, Reflect)]
pub struct Collectible;

#[derive(Bundle)]
pub struct PowerUpServerBundle {
    pub spatial: SpatialBundle,
    pub powerup: PowerUp,
    pub debuff: Debuff,
    
}

#[derive(Bundle)]
pub struct PowerUpClientBundle {
    pub sensor: Sensor,
    pub shape_bundle: LyonRenderBundle,
    pub physics: PhysicsBundle,
}

pub fn spawn_powerup(commands: &mut Commands, transform: Transform, powerup: PowerUp, debuff: Debuff) -> Entity{
  commands.spawn(PowerUpServerBundle{
    spatial: SpatialBundle { transform, ..Default::default() },
    powerup,
    debuff,
  }).id()
}

pub fn spawn_client_powerup(added: Query<(Entity, &PowerUp, &Transform, &Debuff), Added<Collectible>>){

}

pub fn collect_powerups(
    rapier_context: Res<RapierContext>,
    powerups: Query<(&PowerUp, &Debuff, &Collider, &Transform, Entity), Without<Player>>,
    players: Query<Entity, With<Player>>,
    mut cmds: Commands
) {
    powerups.iter().for_each(|(powerup, debuff, collider, transform, p_ent)| {
        let pos = Vec2::new(transform.translation.x, transform.translation.y);
        rapier_context.intersections_with_shape(
            pos,
            transform.rotation.to_euler(EulerRot::XYZ).2,
            collider,
            QueryFilter::default(),
            |e| {
              if let Ok(entity) = players.get(e){
                cmds.entity(entity).insert(powerup.clone()).insert(debuff.clone());
                cmds.entity(p_ent).despawn_recursive();
                return false;
              }
              true
            },
        );
    });
}

pub struct PowerupPlugin;

impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PowerUp>();
        app.register_type::<Debuff>();
        app.register_type::<Collectible>();
        app.replicate::<Collectible>();
        app.replicate::<PowerUp>();
        app.replicate::<Debuff>();

        app.add_system(collect_powerups.in_set(ServerSet::Authority));
    }
}
