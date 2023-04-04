pub mod commands;
pub mod weapons;

use crate::bundles::lyon_rendering::ship_paths::SHIP_PATH;
use crate::bundles::lyon_rendering::{get_path_from_verts, LyonRenderBundle};
use crate::game_manager::GameState;
use crate::network::NetworkOwner;
use crate::player::commands::PlayerCommands;
use crate::player::weapons::WeaponsPlugin;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_prototype_lyon::draw::Stroke;
use bevy_prototype_lyon::prelude::ShapeBundle;
use bevy_rapier2d::prelude::Velocity;
use bevy_replicon::prelude::*;
use bevy_replicon::renet::{RenetClient, ServerEvent};
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default());
        app.add_plugin(WeaponsPlugin);
        app.register_type::<PlayerColor>();
        app.register_type::<Player>();
        app.add_systems(
            (player_actions, spawn_player_on_connected).in_set(OnUpdate(GameState::Playing)),
        );
        app.add_systems((pregame_listen_for_player_connect,).in_set(OnUpdate(GameState::PreGame)));
        app.add_system(insert_player_bundle);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Serialize, Deserialize, Reflect)]
pub enum PlayerAction {
    TurnLeft,
    TurnRight,
    Shoot,
    Thrust,
}

impl PlayerAction {
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();
        input_map.insert(KeyCode::A, Self::TurnLeft);
        input_map.insert(KeyCode::Left, Self::TurnLeft);
        input_map.insert(KeyCode::Right, Self::TurnRight);
        input_map.insert(KeyCode::D, Self::TurnRight);
        input_map.insert(
            SingleAxis::negative_only(GamepadAxisType::LeftStickX, 0.1),
            Self::TurnLeft,
        );
        input_map.insert(
            SingleAxis::positive_only(GamepadAxisType::LeftStickX, 0.1),
            Self::TurnRight,
        );
        input_map.insert(
            SingleAxis::positive_only(GamepadAxisType::LeftStickY, 0.1),
            Self::Thrust,
        );
        input_map.insert(KeyCode::W, Self::Thrust);
        input_map.insert(KeyCode::Up, Self::Thrust);
        input_map.insert(KeyCode::Space, Self::Shoot);
        input_map.insert(GamepadButtonType::South, Self::Shoot);
        input_map.insert(GamepadButtonType::RightTrigger2, Self::Shoot);
        input_map
    }
}

#[derive(Component, Default, Reflect, Copy, Clone)]
#[reflect(Component, Default)]
pub struct Player {
    color: PlayerColor,
}

#[derive(Default, Copy, Clone, Debug, Reflect)]
#[reflect(Default)]
pub enum PlayerColor {
    #[default]
    Red,
    Blue,
    Green,
    Yellow,
}

impl PlayerColor {
    pub fn color(&self) -> Color {
        match self {
            PlayerColor::Red => Color::RED,
            PlayerColor::Blue => Color::BLUE,
            PlayerColor::Green => Color::GREEN,
            PlayerColor::Yellow => Color::YELLOW,
        }
    }

    pub fn get(player_index: usize) -> Self {
        match player_index {
            0 => PlayerColor::Red,
            1 => PlayerColor::Green,
            2 => PlayerColor::Blue,
            3 => PlayerColor::Yellow,
            _ => {
                warn!("Should probably add more colors");
                PlayerColor::Red
            }
        }
    }
}

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    lyon: LyonRenderBundle,
    replication: Replication,
    action_state: ActionState<PlayerAction>,
}

impl PlayerBundle {
    fn with_color(color: PlayerColor) -> Self {
        Self {
            lyon: LyonRenderBundle {
                shape_render: ShapeBundle {
                    path: get_path_from_verts(SHIP_PATH.to_vec(), Vec2::splat(32.)),
                    transform: Transform::from_xyz(
                        0.0,
                        0.0,
                        // Add an offset to prevent z-fighting
                        0.5 + ((color as usize) as f32 * 0.01),
                    ),

                    ..default()
                },
                stroke: Stroke::new(color.color(), 3.0),
                ..default()
            },
            ..default()
        }
    }
}

/// Handle Player connection while in game
fn spawn_player_on_connected(
    mut commands: Commands,
    mut events: EventReader<ServerEvent>,
    player_query: Query<With<Player>>,
) {
    for event in events.iter() {
        if let ServerEvent::ClientConnected(client_id, _) = event {
            let new_player_index = player_query.iter().count();
            commands.spawn_player(PlayerColor::get(new_player_index), NetworkOwner(*client_id));

            info!("Player connected while in play state. Spawning Player")
        }
    }
}

pub fn pregame_listen_for_player_connect(
    mut events: EventReader<ServerEvent>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for event in events.iter() {
        if let ServerEvent::ClientConnected(_client_id, _) = event {
            if game_state.0 != GameState::Playing {
                next_game_state.set(GameState::Playing);
            }

            info!("Player Connected in Pregame! Advancing to play state");
        }
    }
}

/// Handles inserting the player bundle whenever [`Player`] is added to an entity.
fn insert_player_bundle(
    mut commands: Commands,
    query: Query<(Entity, &Player, &NetworkOwner), Added<Player>>,
    client: Option<Res<RenetClient>>,
) {
    for (entity, player, client_id) in query.iter() {
        info!("Inserting Player bundle for new player");
        let player_entity = commands
            .entity(entity)
            .insert(PlayerBundle::with_color(player.color))
            .id();

        if let Some(client) = &client {
            // If we are the client this player is for, add an input map
            if client_id.0 == client.client_id() {
                commands
                    .entity(player_entity)
                    .insert(PlayerAction::default_input_map());
            }
        } else if client_id.0 == SERVER_ID {
            // If we are the server and this player is controlled on the server add an input map
            commands
                .entity(player_entity)
                .insert(PlayerAction::default_input_map());
        }
    }
}

pub fn player_actions(
    mut query: Query<(&Transform, &ActionState<PlayerAction>, &mut Velocity), With<Player>>,
    time: Res<Time>,
) {
    for (transform, action_state, mut velocity) in query.iter_mut() {
        if action_state.pressed(PlayerAction::Thrust) {
            let forward = transform.up();
            velocity.linvel += forward.xy() * time.delta_seconds() * 50.0;
        }
        if action_state.pressed(PlayerAction::TurnRight) {
            velocity.angvel -= 7.0 * time.delta_seconds();
        }

        if action_state.pressed(PlayerAction::TurnLeft) {
            velocity.angvel += 7.0 * time.delta_seconds();
        }
    }
}

#[derive(Component, Default)]
pub struct ReplicatedTransform {
    pub translation: Vec3,
    pub rotation: Quat,
}

pub fn update_replication_transforms(
    mut transforms: Query<(&Transform, &mut ReplicatedTransform)>,
) {
    transforms.iter_mut().for_each(|(trans, mut repl)| {
        repl.translation = trans.translation;
        repl.rotation = trans.rotation;
    });
}

pub fn update_transforms_from_replication(
    mut transforms: Query<(&mut Transform, &ReplicatedTransform)>,
) {
    transforms.iter_mut().for_each(|(mut trans, repl)| {
        trans.translation = repl.translation;
        trans.rotation = repl.rotation;
    });
}

pub struct TransformReplicationPlugin;

impl Plugin for TransformReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            update_replication_transforms,
            update_transforms_from_replication,
        ));
    }
}
