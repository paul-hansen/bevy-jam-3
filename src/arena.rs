use crate::bundles::lyon_rendering::{get_path_from_verts, LyonRenderBundle, UNIT_SQUARE_PATH};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{Path, ShapeBundle};
use bevy_rapier2d::{
    prelude::{Collider, QueryFilter, RapierContext, Sensor},
    rapier::prelude::{CollisionEvent, ContactForceEvent},
};
use bevy_replicon::replication_core::AppReplicationExt;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct ArenaResident {
    pub is_outside: bool,
    pub time_exited: f32,
}

#[derive(
    Component, Reflect, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum Force {
    #[default]
    Red,
    Yellow,
    Blue,
    Pink,
    Green,
    None,
}

///Bundle to be replicated over the wire, Only the server should spawn these
///Any components in here should be .replicate::<ThisType>() and should contain
///all the info needed for the corresponding EnrichBundle to hydrate on clients.
#[derive(Component, Default, Debug, Reflect, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct Arena {
    pub starting_size: Vec2,
    pub current_size: Vec2,
    pub time_spawned: f32,
    pub friendly_force: Force,
}

#[derive(Component)]
pub struct ArenaBoundary;

pub fn spawn_arena(mut cmds: Commands, arenas: Query<(&Arena, Entity), Added<Arena>>) {
    arenas.iter().for_each(|(arena, ent)| {
        let id = cmds
            .spawn(LyonRenderBundle {
                shape_render: ShapeBundle {
                    path: get_path_from_verts(&UNIT_SQUARE_PATH, arena.starting_size),
                    ..default()
                },
                ..default()
            })
            .insert(Name::new("Arena Boundary"))
            .insert(ArenaBoundary)
            .id();

        cmds.entity(ent)
            .insert(SpatialBundle::from_transform(Transform::from_xyz(
                0.0, 0.0, 0.0,
            )))
            .insert((
                Collider::cuboid(arena.starting_size.x / 2.0, arena.starting_size.y / 2.0),
                Sensor,
            ))
            .add_child(id);
    });
}

pub fn update_arena_size(
    mut arenas: Query<(&mut Collider, &Arena), Changed<Arena>>,
    mut boundary_walls: Query<&mut Path, With<ArenaBoundary>>,
) {
    arenas.iter_mut().for_each(|(mut collider, arena)| {
        let Ok(mut wall) = boundary_walls.get_single_mut() else {
            warn!("Could not find boundary wall");
            return;
        };

        *wall = get_path_from_verts(&UNIT_SQUARE_PATH, arena.current_size);
        *collider = Collider::cuboid(arena.current_size.x / 2.0, arena.current_size.y / 2.0);
    });
}

pub fn shrink_arena(mut arenas: Query<&mut Arena>, time: Res<Time>) {
    arenas.iter_mut().for_each(|mut arena| {
        arena.current_size = (arena.current_size
            - (1.0 / 120.0 * time.delta_seconds() * arena.starting_size))
            .max(Vec2::splat(0.001));
    });
}

/* A system that displays the events. */
fn check_arena_residency(
    mut arena_residents: Query<&mut ArenaResident>,
    arenas: Query<(&Arena, &Collider, &Transform)>,
    rapier_context: Res<RapierContext>,
    time: Res<Time>,
) {
    let Ok((_arena, collider, transform)) = arenas.get_single() else {
        debug!("No Arena Found");
        return;
    };

    let pos = Vec2::new(transform.translation.x, transform.translation.y);

    arena_residents.iter_mut().for_each(|mut arena_resident| {
        //If the resident is not outside set the time_exited to the current time, otherwise don't update it after they're already outside
        if !arena_resident.is_outside {
            arena_resident.time_exited = time.elapsed_seconds_wrapped();
        }

        //Set all residents to outside until the collider check overrides it
        arena_resident.is_outside = true;
    });

    rapier_context.intersections_with_shape(
        pos,
        transform.rotation.xyz().z,
        collider,
        QueryFilter::default(),
        |ent| {
            let Ok(mut arena_resident) = arena_residents.get_mut(ent) else {
        return true;
      };
            arena_resident.is_outside = false;
            true
        },
    );
}

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Arena>();
        app.register_type::<Force>();
        app.replicate::<Arena>();
        app.add_systems((spawn_arena, shrink_arena, update_arena_size));

        app.add_event::<ContactForceEvent>();
        app.add_event::<CollisionEvent>();
        app.add_system(check_arena_residency);
    }
}
