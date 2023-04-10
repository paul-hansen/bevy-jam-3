use bevy::prelude::*;
use bevy_prototype_lyon::prelude::ShapeBundle;
use bevy_rapier2d::prelude::{Collider, QueryFilter, RapierContext, Sensor};
use bevy_replicon::replication_core::{AppReplicationExt, Replication};

use crate::{
    bundles::{
        lyon_rendering::{get_path_from_verts, powerups::RAPIDFIRE_PATH, LyonRenderBundle},
        PhysicsBundle,
    },
    game_manager::GameState,
    network::is_server,
    player::{
        weapons::{Weapon, WeaponType},
        Player,
    },
};

#[derive(Component, Reflect, Copy, Default, FromReflect, Debug, Clone)]
#[reflect(Component, Default)]
pub enum PowerUp {
    HitherThither,
    TripleShot,
    #[default]
    RapidFire,
    Shield,
}

#[derive(Component, Default, Copy, FromReflect, Reflect, Debug, Clone)]
#[reflect(Component, Default)]
pub enum Debuff {
    #[default]
    Slowed,
    Inaccuracy,
    HealthBurn,
    ReversedControls,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]

pub struct Collectible;

#[derive(Bundle, Default)]
pub struct PowerUpServerBundle {
    pub spatial: SpatialBundle,
    pub powerup: PowerUp,
    pub debuff: Debuff,
    pub collectible: Collectible,
    pub replication: Replication,
}

#[derive(Bundle, Default)]
pub struct PowerUpClientBundle {
    pub sensor: Sensor,
    pub shape_bundle: LyonRenderBundle,
    pub physics: PhysicsBundle,
}

pub fn spawn_powerup(
    commands: &mut Commands,
    transform: Transform,
    powerup: PowerUp,
    debuff: Debuff,
) -> Entity {
    commands
        .spawn(PowerUpServerBundle {
            spatial: SpatialBundle {
                transform,
                ..Default::default()
            },
            powerup,
            debuff,
            ..default()
        })
        .insert(Name::new("Powerup"))
        .id()
}

pub fn spawn_client_powerup(
    mut cmds: Commands,
    added: Query<(Entity, &Transform), Added<Collectible>>,
) {
    added.iter().for_each(|(ent, transform)| {
        info!("Spawning Client Powerup");
        cmds.entity(ent)
            .insert(PowerUpClientBundle {
                shape_bundle: LyonRenderBundle {
                    shape_render: ShapeBundle {
                        path: get_path_from_verts(RAPIDFIRE_PATH.as_ref(), Vec2::splat(32.)),
                        transform: *transform,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Name::new("Powerup"));
    });
}

pub fn collect_powerups(
    rapier_context: Res<RapierContext>,
    powerups: Query<(&PowerUp, &Debuff, &Collider, &Transform, Entity), Without<Player>>,
    mut players: Query<(Entity, &mut Player, &mut Weapon), With<Player>>,
    mut cmds: Commands,
) {
    powerups
        .iter()
        .for_each(|(powerup, debuff, collider, transform, powerup_ent)| {
            let pos = Vec2::new(transform.translation.x, transform.translation.y);
            rapier_context.intersections_with_shape(
                pos,
                transform.rotation.to_euler(EulerRot::XYZ).2,
                collider,
                QueryFilter::default(),
                |e| {
                    if let Ok((_entity, mut player, mut weapon)) = players.get_mut(e) {
                        player.powerup = Some(*powerup);
                        player.debuff = Some(*debuff);
                        *weapon = Weapon {
                            weapon_type: WeaponType::Laser { fire_rate: 10.0 },
                            ..Default::default()
                        };
                        cmds.entity(powerup_ent).despawn_recursive();
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
        app.register_type::<Option<PowerUp>>();

        app.register_type::<Option<Debuff>>();
        app.register_type::<Debuff>();
        app.register_type::<Collectible>();
        app.replicate::<Collectible>();
        app.replicate::<PowerUp>();
        app.replicate::<Debuff>();

        app.add_system(spawn_client_powerup);
        app.add_system(
            collect_powerups
                .run_if(is_server())
                .in_set(OnUpdate(GameState::Playing)),
        );
    }
}
