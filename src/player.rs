use crate::bundles::lyon_rendering::ship_paths::SHIP_PATH;
use crate::bundles::lyon_rendering::{get_path_from_verts, LyonRenderBundle};
use bevy::prelude::*;
use bevy_prototype_lyon::draw::Stroke;
use bevy_prototype_lyon::prelude::ShapeBundle;
use bevy_replicon::prelude::Replication;
use bevy_replicon::renet::{RenetClient, ServerEvent};
use leafwing_input_manager::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default());
        app.register_type::<PlayerColor>();
        app.register_type::<Player>();
        app.register_type::<Option<u64>>();
        app.add_system(insert_player_bundle);
        app.add_system(player_actions);
        app.add_system(spawn_player_on_connected);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum PlayerAction {
    Aim,
    Shoot,
    Thrust,
}

impl PlayerAction {
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();
        input_map.insert(VirtualDPad::wasd(), Self::Aim);
        input_map.insert(DualAxis::left_stick(), Self::Aim);
        input_map
    }
}

#[derive(Component, Default, Reflect, Copy, Clone)]
#[reflect(Component, Default)]
pub struct Player {
    color: PlayerColor,
    client_id: Option<u64>,
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

#[derive(Bundle)]
pub struct PlayerBundle {
    lyon: LyonRenderBundle,
    replication: Replication,
}

impl PlayerBundle {
    fn with_color(color: PlayerColor) -> Self {
        Self {
            lyon: LyonRenderBundle {
                shape_render: ShapeBundle {
                    path: get_path_from_verts(SHIP_PATH.to_vec(), 32.),
                    transform: Transform::from_xyz(0.0, 0.0, 0.5),

                    ..default()
                },
                stroke: Stroke::new(color.color(), 3.0),
                ..default()
            },
            replication: Replication,
        }
    }
}

/// Server only
pub fn spawn_player(color: PlayerColor, cmds: &mut Commands, client_id: Option<u64>) -> Entity {
    debug!("Spawning player");
    return cmds
        .spawn((
            Player { color, client_id },
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
                Some(*client_id),
            );
        }
    }
}

/// Handles inserting the player bundle whenever [`Player`] is added to an entity.
fn insert_player_bundle(
    mut commands: Commands,
    query: Query<(Entity, &Player), Added<Player>>,
    client: Option<Res<RenetClient>>,
) {
    for (entity, player) in query.iter() {
        info!("Inserting Player bundle for new player");
        let player_entity = commands
            .entity(entity)
            .insert(PlayerBundle::with_color(player.color))
            .id();

        if let Some(client) = &client {
            // If we are the client this player is for, add an input map
            if player.client_id == Some(client.client_id()) {
                commands
                    .entity(player_entity)
                    .insert(PlayerAction::default_input_map());
            }
        } else if client.is_none() && player.client_id.is_none() {
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
        if let Some(move_direction) = action_state.axis_pair(PlayerAction::Aim) {
            if move_direction.length_squared() > 0.01 {
                transform.translation +=
                    move_direction.xy().extend(0.0) * time.delta_seconds() * 100.0;
            }
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
