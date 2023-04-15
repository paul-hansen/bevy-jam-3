pub mod commands;
pub mod weapons;

use crate::arena::ArenaResident;
use crate::bundles::lyon_rendering::ship_parts::THRUSTER_JET;
use crate::bundles::lyon_rendering::ship_paths::SHIP_PATH;
use crate::bundles::lyon_rendering::{get_path_from_verts, LyonRenderBundle};
use crate::bundles::PhysicsBundle;
use crate::game_manager::GameState;
use crate::network::{is_server, NetworkOwner};
use crate::player::commands::PlayerCommands;
use crate::player::weapons::WeaponsPlugin;
use crate::powerup::{Debuff, PowerUp};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_prototype_lyon::draw::Stroke;
use bevy_prototype_lyon::prelude::{Fill, ShapeBundle};
use bevy_rapier2d::prelude::{Damping, Velocity};
use bevy_replicon::prelude::*;
use bevy_replicon::renet::{RenetClient, ServerEvent};
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::fmt::Formatter;

use self::weapons::DamagedEvent;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default());
        app.add_plugin(WeaponsPlugin);
        app.register_type::<PlayerColor>();
        app.register_type::<Players>();
        app.register_type::<Thruster>();
        app.insert_resource(Players::default());
        app.register_type::<Player>();
        app.add_systems(
            (player_actions, damage_players_outside_arena).in_set(OnUpdate(GameState::Playing)),
        );
        app.add_system(update_thruster.run_if(is_server()));
        app.add_system(spawn_player_on_connected);
        app.add_system(despawn_on_player_disconnect);
        app.add_systems((pregame_listen_for_player_connect,).in_set(OnUpdate(GameState::PreGame)));
        app.add_system(insert_player_bundle);
        app.add_system(handle_thruster);
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
    pub color: PlayerColor,
    pub powerup: Option<PowerUp>,
    pub debuff: Option<Debuff>,
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.color)
    }
}

#[derive(
    Default, Copy, Clone, Debug, Reflect, Serialize, Deserialize, FromReflect, Hash, Eq, PartialEq,
)]
#[reflect(Default)]
pub enum PlayerColor {
    #[default]
    Red,
    Blue,
    Green,
    Purple,
    Cyan,
    Orange,
}

impl std::fmt::Display for PlayerColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl PlayerColor {
    pub const MAX: usize = 5;
    pub fn color(&self) -> Color {
        match self {
            PlayerColor::Red => Color::RED,
            PlayerColor::Blue => Color::BLUE,
            PlayerColor::Green => Color::GREEN,
            PlayerColor::Purple => Color::PURPLE,
            PlayerColor::Cyan => Color::CYAN,
            PlayerColor::Orange => Color::ORANGE,
        }
    }

    pub fn get(player_index: usize) -> Self {
        match player_index {
            0 => PlayerColor::Red,
            1 => PlayerColor::Blue,
            2 => PlayerColor::Green,
            3 => PlayerColor::Purple,
            4 => PlayerColor::Cyan,
            5 => PlayerColor::Orange,
            _ => {
                warn!("Should probably add more colors");
                PlayerColor::Red
            }
        }
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource, Default)]
pub struct Players {
    colors: HashMap<u64, PlayerColor>,
    clients: HashMap<PlayerColor, u64>,
}

impl Players {
    pub fn color(&self, client_id: u64) -> Option<PlayerColor> {
        self.colors.get(&client_id).copied()
    }

    #[allow(dead_code)]
    pub fn client(&self, color: PlayerColor) -> Option<u64> {
        self.clients.get(&color).copied()
    }

    pub fn insert(&mut self, color: PlayerColor, client_id: u64) {
        self.clients.insert(color, client_id);
        self.colors.insert(client_id, color);
    }

    #[allow(dead_code)]
    pub fn remove_color(&mut self, color: PlayerColor) {
        if let Some(id) = self.clients.remove(&color) {
            self.colors.remove(&id);
        }
    }
    pub fn remove_client(&mut self, client_id: u64) {
        if let Some(color) = self.colors.remove(&client_id) {
            self.clients.remove(&color);
        }
    }

    fn available_color(&self) -> Option<PlayerColor> {
        for x in 0..=PlayerColor::MAX {
            let color = PlayerColor::get(x);
            if self.clients.get(&color).is_none() {
                return Some(color);
            }
        }
        None
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    name: Name,
    lyon: LyonRenderBundle,
    replication: Replication,
    action_state: ActionState<PlayerAction>,
    physics: PhysicsBundle,
    arena_resident: ArenaResident,
}

impl PlayerBundle {
    fn with_color(color: PlayerColor) -> Self {
        Self {
            name: Name::new(format!("Player {:?}", color)),
            lyon: LyonRenderBundle {
                shape_render: ShapeBundle {
                    path: get_path_from_verts(&SHIP_PATH, Vec2::splat(32.)),
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
            physics: PhysicsBundle {
                damping: Damping {
                    linear_damping: 0.4,
                    angular_damping: 1.0,
                },
                ..default()
            },
            ..default()
        }
    }
}

#[derive(Component, Default, Reflect, FromReflect)]
#[reflect(Default, Component)]
pub struct Thruster {
    pub val: f32,
}

#[derive(Component, Default, Reflect)]
#[reflect(Default, Component)]
pub struct ScaleYFromThruster;

#[derive(Bundle, Default)]
pub struct ThrusterVisualsBundle {
    lyon_render: LyonRenderBundle,
    marker: ScaleYFromThruster,
}

impl ThrusterVisualsBundle {
    pub fn with_color(p_color: PlayerColor) -> Self {
        ThrusterVisualsBundle {
            lyon_render: LyonRenderBundle {
                shape_render: ShapeBundle {
                    path: get_path_from_verts(&THRUSTER_JET, Vec2::splat(24.0)),
                    transform: Transform::from_rotation(Quat::from_rotation_z(PI))
                        .with_translation(Vec3::new(0.0, -24.0, 0.0)),
                    ..default()
                },
                stroke: Stroke::color(p_color.color()),
                fill: Fill::color(p_color.color()),
            },
            marker: ScaleYFromThruster,
        }
    }
}

fn handle_thruster(
    mut scale_y_from_thrust: Query<(Entity, &mut Transform), With<ScaleYFromThruster>>,
    thrusters: Query<&Thruster>,
    parents: Query<&Parent>,
    time: Res<Time>,
) {
    for (entity, mut transform) in scale_y_from_thrust.iter_mut() {
        for thruster in thrusters.iter_many(parents.iter_ancestors(entity)) {
            transform.scale.y = thruster.val
                * ((time.elapsed_seconds_wrapped() * 20.0)
                    .sin()
                    .abs()
                    .clamp(0.5, 1.0));
        }
    }
}

/// Handle Player connection while in game
fn spawn_player_on_connected(
    mut commands: Commands,
    mut events: EventReader<ServerEvent>,
    players: Res<Players>,
) {
    for event in events.iter() {
        if let ServerEvent::ClientConnected(client_id, _) = event {
            commands.spawn_player(
                players.available_color().unwrap_or_default(),
                NetworkOwner(*client_id),
            );

            info!("Player connected while in play state. Spawning Player")
        }
    }
}

fn despawn_on_player_disconnect(
    mut commands: Commands,
    mut events: EventReader<ServerEvent>,
    mut players: ResMut<Players>,
    query: Query<(Entity, &Player)>,
) {
    for event in events.iter() {
        if let ServerEvent::ClientDisconnected(client_id) = event {
            if let Some(color) = players.color(*client_id) {
                for (entity, player) in query.iter() {
                    if player.color == color {
                        commands.entity(entity).despawn_recursive();
                    }
                }

                info!("Player {color} disconnected");
            }
            players.remove_client(*client_id);
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

pub fn damage_players_outside_arena(
    server: Option<Res<RenetServer>>,
    players: Query<(&ArenaResident, Entity), With<Player>>,
    mut dmg_events: EventWriter<DamagedEvent>,
    time: Res<Time>,
) {
    if server.is_none() {
        return;
    }

    players.iter().for_each(|(arena_resident, entity)| {
        if arena_resident.is_outside {
            dmg_events.send(DamagedEvent {
                entity,
                amount: 34.0 * time.delta().as_secs_f32(),
                normal: None,
                direction: None,
                point: None,
            })
        }
    })
}

/// Handles inserting the player bundle whenever [`Player`] is added to an entity.
fn insert_player_bundle(
    mut commands: Commands,
    query: Query<(Entity, &Player, &NetworkOwner, &Transform), Added<Player>>,
    client: Option<Res<RenetClient>>,
) {
    for (entity, player, client_id, transform) in query.iter() {
        info!("Inserting Player bundle for player: {}", player);
        commands.entity(entity).with_children(|cb| {
            cb.spawn(ThrusterVisualsBundle::with_color(player.color));
        });

        let player_entity = commands
            .entity(entity)
            .insert((PhysicsBundle::default(),))
            .insert({
                let mut bundle = PlayerBundle::with_color(player.color);
                bundle.lyon.shape_render.transform = *transform;
                bundle
            })
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

pub fn update_thruster(
    mut query: Query<(&Player, &ActionState<PlayerAction>, &mut Thruster), With<Player>>,
    time: Res<Time>,
) {
    for (player, action_state, mut thruster) in query.iter_mut() {
        if action_state.pressed(PlayerAction::Thrust) && player.debuff != Some(Debuff::Slowed) {
            thruster.val = (thruster.val + time.delta_seconds()).min(1.0).max(0.0);
        } else {
            thruster.val = (thruster.val - time.delta_seconds()).min(1.0).max(0.0);
        }
    }
}

pub fn player_actions(
    mut query: Query<
        (
            &Player,
            &Transform,
            &ActionState<PlayerAction>,
            &mut Velocity,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    for (player, transform, action_state, mut velocity) in query.iter_mut() {
        if action_state.pressed(PlayerAction::Thrust) && player.debuff != Some(Debuff::Slowed) {
            let forward = transform.up();
            velocity.linvel += forward.xy() * time.delta_seconds() * 50.0;
        }

        if action_state.pressed(PlayerAction::TurnRight) {
            velocity.angvel = -700.0 * time.delta_seconds();
        } else if action_state.pressed(PlayerAction::TurnLeft) {
            velocity.angvel = 700.0 * time.delta_seconds();
        } else {
            velocity.angvel = 0.0;
        }
    }
}
