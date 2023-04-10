use crate::bundles::lyon_rendering::projectile_paths::LASER_PATH;
use crate::bundles::lyon_rendering::{get_path_from_verts, LyonRenderBundle};
use crate::game_manager::GameState;
use crate::network::{is_server, NetworkOwner};
use crate::player::PlayerAction;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_kira_audio::AudioControl;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::render::ShapeMaterial;
use bevy_rapier2d::plugin::RapierContext;
use bevy_rapier2d::prelude::{ExternalImpulse, QueryFilter};
use bevy_replicon::prelude::{AppReplicationExt, Replication};
use bevy_replicon::server::ServerSet;
use leafwing_input_manager::action_state::ActionState;
use rand::{thread_rng, Rng};
use std::cmp::Ordering;
use std::f32::consts::TAU;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamagedEvent>();
        app.register_type::<Laser>();
        app.register_type::<WeaponType>();
        app.replicate::<Weapon>();
        app.replicate::<Laser>();
        app.add_system(
            fire_weapon_action
                .run_if(is_server())
                .in_set(OnUpdate(GameState::Playing)),
        );
        app.add_system(move_lasers);
        app.add_system(detect_laser_hits.in_set(ServerSet::Authority));
        app.add_system(
            despawn_oldest_if_exceed_count::<30, Laser>
                .run_if(is_server())
                .in_base_set(CoreSet::PostUpdate),
        );
        app.add_system(
            despawn_after_milliseconds::<800, Laser>
                .run_if(is_server())
                .in_base_set(CoreSet::PostUpdate),
        );
        app.add_system(spawn_bundle_on_laser_added.in_base_set(CoreSet::PreUpdate));
    }
}

pub fn spawn_bundle_on_laser_added(
    mut commands: Commands,
    query: Query<Entity, Added<Laser>>,
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: ResMut<AssetServer>,
) {
    for entity in query.iter() {
        let Some(mut entcmds) = commands.get_entity(entity) else {
            warn!("Could not find entity to insert bundle into");
            return;
        };

        audio.play(asset_server.load("laserShoot.mp3")).with_playback_rate(thread_rng().gen_range(0.75 .. 1.125));

        entcmds.insert(LaserBundle::default());
    }
}

#[derive(Debug)]
pub struct DamagedEvent {
    pub entity: Entity,
    pub amount: f32,
    /// The normal on the surface of this object at the point of impact
    pub normal: Option<Vec2>,
    /// The direction the damage is coming from
    pub direction: Option<Vec2>,
    pub point: Option<Vec2>,
}

fn fire_weapon_action(
    mut commands: Commands,
    mut query: Query<(
        &mut Weapon,
        &ActionState<PlayerAction>,
        &GlobalTransform,
        &NetworkOwner,
    )>,
    time: Res<Time>,
) {
    for (mut weapon, action_state, transform, owner) in query.iter_mut() {
        if action_state.pressed(PlayerAction::Shoot) {
            weapon.fire(
                &mut commands,
                transform.compute_transform(),
                time.as_ref(),
                owner,
            );
        }
    }
}

#[derive(Component, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default)]
pub enum WeaponType {
    Laser {
        /// How often this can fire per second
        fire_rate: f32,
    },
    Scattergun {
        fire_rate: f32,
        count: u8,
    }
}

impl Default for WeaponType {
    fn default() -> Self {
        Self::Laser { fire_rate: 10.0 }
    }
}

#[derive(Component, Reflect, Default, Copy, Clone, Debug)]
#[reflect(Component, Default)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub last_fire: f32,
}

impl Weapon {
    pub fn fire(
        &mut self,
        commands: &mut Commands,
        transform: Transform,
        time: &Time,
        owner: &NetworkOwner,
    ) {
        match self.weapon_type {
            WeaponType::Laser { fire_rate } => {
                let seconds_between_fire = 1.0 / fire_rate;
                let time_since_last_fire = time.elapsed_seconds_wrapped() - self.last_fire;

                if time_since_last_fire > seconds_between_fire {
                    info!("Shoot Laser");
                    commands.spawn((
                        Name::new("Laser"),
                        Replication,
                        Laser,
                        *owner,
                        SpatialBundle::from_transform(transform),
                        SpawnTime(time.elapsed_seconds_wrapped()),
                    ));
                    self.last_fire = time.elapsed_seconds_wrapped();
                }
            },
            WeaponType::Scattergun { fire_rate, count } => {
                let seconds_between_fire = 1.0 / fire_rate;
                let time_since_last_fire = time.elapsed_seconds_wrapped() - self.last_fire;

                if time_since_last_fire > seconds_between_fire {
                    info!("Shooting Scattergun");
                    let mut rng = thread_rng();
                    let mut t = transform;
                    let mut rot;

                    for _ in 0.. count {
                        rot = rng.gen_range(-TAU/3.0 .. TAU/3.0);
                        t.rotate_z(rot);

                        commands.spawn((
                            Name::new("Laser"),
                            Replication,
                            Laser,
                            *owner,
                            SpatialBundle::from_transform(t),
                            SpawnTime(time.elapsed_seconds_wrapped()),
                        ));

                        t.rotate_z(-rot);
                    }

                    self.last_fire = time.elapsed_seconds_wrapped();
                }
            }
        }
    }
}

#[derive(Component, Reflect, Default, Copy, Clone, Debug)]
#[reflect(Component, Default)]
pub struct Laser;

impl Laser {
    pub const UNITS_PER_SECOND: f32 = 1000.0;
    pub const DAMAGE: f32 = 50.0;
    /// How much impulse should be applied to an object the laser hits?
    pub const DAMAGE_IMPULSE: f32 = 50.0;
}

fn move_lasers(mut query: Query<&mut Transform, With<Laser>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        let forward = transform.up();
        transform.translation += forward * time.delta_seconds() * Laser::UNITS_PER_SECOND;
    }
}

fn detect_laser_hits(
    mut commands: Commands,
    query: Query<(Entity, &GlobalTransform, &NetworkOwner), With<Laser>>,
    rapier_context: Res<RapierContext>,
    time: Res<Time>,
    network_owners: Query<&NetworkOwner>,
    mut damaged_events: EventWriter<DamagedEvent>,
    mut impulses: Query<&mut ExternalImpulse>,
) {
    for (laser_entity, transform, owner) in query.iter() {
        // Start from where the laser would have been the last frame
        let ray_start = transform.translation().xy()
            - (transform.down().xy() * time.delta_seconds() * Laser::UNITS_PER_SECOND);
        if let Some((hit_entity, intersection)) = rapier_context.cast_ray_and_get_normal(
            ray_start,
            transform.up().xy(),
            Laser::UNITS_PER_SECOND * time.delta_seconds(),
            true,
            QueryFilter::default().exclude_sensors(),
        ) {
            if network_owners.get(hit_entity) != Ok(owner) {
                damaged_events.send(DamagedEvent {
                    entity: hit_entity,
                    amount: Laser::DAMAGE,
                    normal: Some(intersection.normal),
                    direction: Some(transform.up().xy()),
                    point: Some(intersection.point),
                });
                if let Ok(mut impulse) = impulses.get_mut(hit_entity) {
                    impulse.impulse += transform.up().xy() * Laser::DAMAGE_IMPULSE;
                }
                commands.entity(laser_entity).despawn_recursive();
            }
        }
    }
}

#[derive(Bundle)]
pub struct LaserBundle {
    pub path: Path,
    pub mesh: Mesh2dHandle,
    pub material: Handle<ShapeMaterial>,
    pub stroke: Stroke,
    pub fill: Fill,
    pub visibility: Visibility,
    pub computed: ComputedVisibility,
    pub global_transform: GlobalTransform,
}

impl Default for LaserBundle {
    fn default() -> Self {
        let lyon = LyonRenderBundle {
            shape_render: ShapeBundle {
                path: get_path_from_verts(&LASER_PATH, Vec2::splat(2.0)),
                ..default()
            },
            ..default()
        };
        Self {
            path: lyon.shape_render.path,
            mesh: lyon.shape_render.mesh,
            material: lyon.shape_render.material,
            stroke: lyon.stroke,
            fill: lyon.fill,
            visibility: Default::default(),
            computed: Default::default(),
            global_transform: Default::default(),
        }
    }
}

/// Record of the elapsed time this component was spawned
#[derive(Component, Reflect, Copy, Clone, Debug, PartialOrd, PartialEq)]
#[reflect(Component)]
pub struct SpawnTime(pub f32);

impl FromWorld for SpawnTime {
    fn from_world(world: &mut World) -> Self {
        let time = world.resource::<Time>();
        Self(time.elapsed_seconds_wrapped())
    }
}

/// Limits the number of entities that can exist in the world at a time with the given component.
/// Will despawn the oldest as needed to enforce this.
/// Useful for limiting entities we are replicating like bullets where we don't want too many.
/// Entities must have the [`SpawnTime`] component.
fn despawn_oldest_if_exceed_count<const MAX: usize, C: Component>(
    mut commands: Commands,
    query: Query<(Entity, &SpawnTime), With<C>>,
) {
    if query.iter().count() > MAX {
        let mut all = query.iter().collect::<Vec<_>>();
        all.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(Ordering::Equal));
        all[MAX..]
            .iter()
            .for_each(|(entity, _)| commands.entity(*entity).despawn_recursive());
    }
}

fn despawn_after_milliseconds<const MILLISECONDS: usize, C: Component>(
    mut commands: Commands,
    query: Query<(Entity, &SpawnTime), With<C>>,
    time: Res<Time>,
) {
    for (entity, spawn_time) in query.iter() {
        if ((time.elapsed_seconds_wrapped() - spawn_time.0) * 1000.0) as usize > MILLISECONDS {
            commands.entity(entity).despawn_recursive();
        }
    }
}
