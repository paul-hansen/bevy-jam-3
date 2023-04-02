use crate::bundles::lyon_rendering::ship_paths::SHIP_PATH;
use crate::bundles::lyon_rendering::{get_path_from_verts, LyonRenderBundle};
use crate::network::NetworkOwner;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_prototype_lyon::draw::Stroke;
use bevy_prototype_lyon::prelude::ShapeBundle;
use bevy_replicon::prelude::*;
use bevy_replicon::renet::{RenetClient, ServerEvent};
use leafwing_input_manager::orientation::{Orientation, Rotation};
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::FRAC_2_PI;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default());
        app.register_type::<PlayerColor>();
        app.register_type::<Player>();
        app.add_system(insert_player_bundle);
        app.add_system(player_actions);
        app.add_system(spawn_player_on_connected);
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
                    path: get_path_from_verts(SHIP_PATH.to_vec(), 32.),
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

/// Server only
pub fn spawn_player(color: PlayerColor, commands: &mut Commands, client_id: u64) -> Entity {
    debug!("Spawning player");
    return commands
        .spawn((
            Player { color },
            NetworkOwner(client_id),
            Replication,
            ActionState::<PlayerAction>::default(),
        ))
        .id();
}

fn spawn_player_on_connected(
    mut commands: Commands,
    mut events: EventReader<ServerEvent>,
    player_query: Query<With<Player>>,
) {
    for event in events.iter() {
        if let ServerEvent::ClientConnected(client_id, _) = event {
            let new_player_index = player_query.iter().count();

            spawn_player(
                PlayerColor::get(new_player_index),
                &mut commands,
                *client_id,
            );
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
    mut query: Query<(&mut Transform, &ActionState<PlayerAction>), With<Player>>,
    time: Res<Time>,
) {
    for (mut transform, action_state) in query.iter_mut() {
        if action_state.pressed(PlayerAction::Thrust) {
            let forward = transform.up();
            transform.translation += forward.xy().extend(0.0) * time.delta_seconds() * 100.0;
        }
        if action_state.pressed(PlayerAction::TurnRight) {
            let right =
                Quat::from_rotation_z(transform.rotation.to_euler(EulerRot::XYZ).2 - FRAC_2_PI);

            transform.rotation.rotate_towards(
                right,
                Some(Rotation::from_degrees(90.0 * time.delta_seconds())),
            );
        }

        if action_state.pressed(PlayerAction::TurnLeft) {
            let left =
                Quat::from_rotation_z(transform.rotation.to_euler(EulerRot::XYZ).2 + FRAC_2_PI);

            transform.rotation.rotate_towards(
                left,
                Some(Rotation::from_degrees(90.0 * time.delta_seconds())),
            );
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
