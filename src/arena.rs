use bevy::prelude::*;
use bevy_prototype_lyon::prelude::ShapeBundle;
use bevy_rapier2d::{prelude::{Collider, Sensor}, rapier::prelude::{CollisionEvent, ContactForceEvent}};
use bevy_replicon::replication_core::AppReplicationExt;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use crate::bundles::lyon_rendering::{get_path_from_verts, LyonRenderBundle, UNIT_SQUARE_PATH};

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

pub fn spawn_arena(mut cmds: Commands, arenas: Query<(&Arena, Entity), Added<Arena>>) {
    arenas.iter().for_each(|(arena, ent)| {
        info!("Enriching spawned Arena");
        let id = cmds
            .spawn(LyonRenderBundle {
                shape_render: ShapeBundle {
                    path: get_path_from_verts(&UNIT_SQUARE_PATH, arena.starting_size),
                    ..default()
                },
                ..default()
            })
            .insert(Name::new("Arena Boundary"))
            .id();

        cmds.entity(ent)
            .insert(SpatialBundle::from_transform(Transform::from_xyz(
                0.0,
                0.0,
                0.0,
            )))
            .insert((Collider::cuboid(arena.starting_size.x/2.0, arena.starting_size.y/2.0), Sensor))
            .add_child(id);
    });
}


lazy_static!{
    pub static ref ARENA_BOUNDARY: Vec<(f32, f32)> =
        vec![(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0)];
}


/* A system that displays the events. */
fn display_events(
  mut collision_events: EventReader<CollisionEvent>,
  mut contact_force_events: EventReader<ContactForceEvent>,
) {
  for collision_event in collision_events.iter() {
      println!("Received collision event: {:?}", collision_event);
  }

  for contact_force_event in contact_force_events.iter() {
      println!("Received contact force event: {:?}", contact_force_event);
  }
}

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Arena>();
        app.register_type::<Force>();
        app.replicate::<Arena>();
        app.add_system(spawn_arena);

        app.add_event::<ContactForceEvent>();
        app.add_event::<CollisionEvent>();
        app.add_system(display_events);
    }
}
