use bevy::prelude::*;
use bevy_replicon::prelude::Replication;
use bevy_replicon::renet::ServerEvent;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerColor>();
        app.register_type::<Player>();
        app.add_system(insert_player_bundle);
        app.add_system(spawn_player_on_connected);
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
    sprite: SpriteBundle,
    player: Player,
    replication: Replication,
}

/// Server only
pub fn spawn_player(color: PlayerColor, cmds: &mut Commands) -> Entity {
    debug!("Spawning player");
    return cmds.spawn((Player { color }, Replication)).id();
}

fn spawn_player_on_connected(
    mut commands: Commands,
    mut events: EventReader<ServerEvent>,
    player_query: Query<With<Player>>,
) {
    for event in events.iter() {
        if let ServerEvent::ClientConnected(_, _) = event {
            let new_player_index = player_query.iter().count();

            spawn_player(PlayerColor::get(new_player_index), &mut commands);
        }
    }
}

/// Handles inserting the player bundle whenever [`Player`] is added to an entity.
fn insert_player_bundle(
    mut commands: Commands,
    query: Query<(Entity, &Player), Added<Player>>,
    asset_server: ResMut<AssetServer>,
) {
    for (entity, player) in query.iter() {
        info!("Inserting Player bundle for new player");
        commands.entity(entity).insert(PlayerBundle {
            player: *player,
            sprite: {
                SpriteBundle {
                    sprite: Sprite {
                        color: player.color.color(),
                        ..default()
                    },
                    texture: asset_server.load("icon.png"),
                    ..default()
                }
            },
            ..default()
        });
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
