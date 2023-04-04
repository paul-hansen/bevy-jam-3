use crate::bundles::PhysicsBundle;
use crate::network::NetworkOwner;
use crate::player::weapons::Weapon;
use crate::player::{Player, PlayerAction, PlayerColor};
use bevy::ecs::system::{Command, Spawn};
use bevy::prelude::{Commands, World};
use bevy_replicon::prelude::Replication;
use leafwing_input_manager::action_state::ActionState;

pub struct SpawnPlayer {
    pub color: PlayerColor,
    pub network_owner: NetworkOwner,
}

impl Command for SpawnPlayer {
    fn write(self, world: &mut World) {
        Spawn {
            bundle: (
                Player { color: self.color },
                self.network_owner,
                Replication,
                ActionState::<PlayerAction>::default(),
                PhysicsBundle::default(),
                Weapon::default(),
            ),
        }
        .write(world);
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
