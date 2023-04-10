use crate::health::Health;
use crate::network::NetworkOwner;
use crate::player::weapons::Weapon;
use crate::player::{Player, PlayerAction, PlayerColor, Players};
use bevy::ecs::system::{Command, Spawn};
use bevy::prelude::*;
use bevy_replicon::prelude::Replication;
use leafwing_input_manager::action_state::ActionState;

pub struct SpawnPlayer {
    pub color: PlayerColor,
    pub network_owner: NetworkOwner,
}

const SPAWN_LOCATIONS: [(Vec2, f32); 6] = [
    (Vec2::new(440.0, 350.0), 135.0),
    (Vec2::new(-440.0, -350.0), -45.0),
    (Vec2::new(440.0, -350.0), 45.0),
    (Vec2::new(-440.0, 350.0), -135.0),
    (Vec2::new(0.0, 350.0), 180.0),
    (Vec2::new(0.0, -350.0), 0.0),
];

impl Command for SpawnPlayer {
    fn write(self, world: &mut World) {
        let (position, rotation) = SPAWN_LOCATIONS[self.color as usize];

        world
            .resource_mut::<Players>()
            .colors
            .insert(self.network_owner.0, self.color);

        Spawn {
            bundle: (
                Player {
                    color: self.color,
                    ..Default::default()
                },
                Health::default(),
                self.network_owner,
                Replication,
                ActionState::<PlayerAction>::default(),
                Weapon {
                    weapon_type: super::weapons::WeaponType::Laser { fire_rate: 1.5 },
                    ..default()
                },
                Transform::from_translation(position.extend(0.0))
                    .with_rotation(Quat::from_rotation_z(rotation.to_radians())),
            ),
        }
        .write(world);

        world
            .resource_mut::<Players>()
            .insert(self.color, self.network_owner.0);
    }
}

pub trait PlayerCommands {
    fn spawn_player(&mut self, color: PlayerColor, network_owner: NetworkOwner);
}

impl<'w, 's> PlayerCommands for Commands<'w, 's> {
    fn spawn_player(&mut self, color: PlayerColor, network_owner: NetworkOwner) {
        self.add(SpawnPlayer {
            color,
            network_owner,
        });
    }
}
