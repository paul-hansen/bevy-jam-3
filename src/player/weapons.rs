use crate::bundles::lyon_rendering::projectile_paths::LASER_PATH;
use crate::bundles::lyon_rendering::{get_path_from_verts, LyonRenderBundle};
use crate::network::util::spawn_bundle_default_on_added;
use crate::player::PlayerAction;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::render::ShapeMaterial;
use bevy_replicon::prelude::{AppReplicationExt, Replication};
use bevy_replicon::server::ServerSet;
use leafwing_input_manager::action_state::ActionState;
use std::cmp::Ordering;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Laser>();
        app.register_type::<WeaponType>();
        app.replicate::<Weapon>();
        app.replicate::<Laser>();
        app.add_system(fire_weapon_action.in_set(ServerSet::Authority));
        app.add_system(update_lasers);
        app.add_system(despawn_oldest_if_exceed_count::<30, Laser>.in_set(ServerSet::Authority));
        app.add_system(despawn_after_milliseconds::<800, Laser>.in_set(ServerSet::Authority));
        app.add_system(spawn_bundle_default_on_added::<Laser, LaserBundle>);
    }
}

fn fire_weapon_action(
    mut commands: Commands,
    mut query: Query<(&mut Weapon, &ActionState<PlayerAction>, &GlobalTransform)>,
    time: Res<Time>,
) {
    for (mut weapon, action_state, transform) in query.iter_mut() {
        if action_state.pressed(PlayerAction::Shoot) {
            weapon.fire(&mut commands, transform.compute_transform(), time.as_ref());
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
}

impl Default for WeaponType {
    fn default() -> Self {
        Self::Laser { fire_rate: 10.0 }
    }
}

#[derive(Component, Reflect, Default, Copy, Clone, Debug)]
#[reflect(Component, Default)]
pub struct Weapon {
    weapon_type: WeaponType,
    last_fire: f32,
}

impl Weapon {
    pub fn fire(&mut self, commands: &mut Commands, transform: Transform, time: &Time) {
        match self.weapon_type {
            WeaponType::Laser { fire_rate } => {
                let seconds_between_fire = 1.0 / fire_rate;
                let time_since_last_fire = time.elapsed_seconds_wrapped() - self.last_fire;

                if time_since_last_fire > seconds_between_fire {
                    debug!("Shoot Laser");
                    commands.spawn((
                        Name::new("Laser"),
                        Replication,
                        Laser,
                        SpatialBundle::from_transform(transform),
                        SpawnTime(time.elapsed_seconds_wrapped()),
                    ));
                    self.last_fire = time.elapsed_seconds_wrapped();
                }
            }
        }
    }
}

#[derive(Component, Reflect, Default, Copy, Clone, Debug)]
#[reflect(Component, Default)]
pub struct Laser;

fn update_lasers(mut query: Query<&mut Transform, With<Laser>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        let forward = transform.up();
        transform.translation += forward * time.delta_seconds() * 1000.0;
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
                path: get_path_from_verts(LASER_PATH.to_vec(), Vec2::splat(2.0)),
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
