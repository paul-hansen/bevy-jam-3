use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::view::NoFrustumCulling;
use bevy_replicon::prelude::{SendMode, ServerEventAppExt, ToClients};
use bevy_replicon::renet::RenetServer;
use bevy_replicon::replication_core::Replication;
use bevy_replicon::server::SERVER_ID;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::network::{is_server, NetworkOwner};
use crate::player::commands::PlayerCommands;
use crate::player::{Player, PlayerColor, Players};
use crate::powerup::{spawn_powerup, Debuff, PowerUp};
use crate::{
    arena::{Arena, Force},
    asteroid::{asteroid_spawn, Asteroid},
    bundles::lyon_rendering::roid_paths::RoidPath,
};

#[derive(Debug, Hash, Eq, PartialEq, Clone, States, Default, Reflect)]
#[reflect(Default)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    PreGame,
    Playing,
    PostGame,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameEvent {
    RoundWon { winner: PlayerColor },
    Tie,
    Restart,
}

pub struct GameManager;

impl Plugin for GameManager {
    fn build(&self, app: &mut App) {
        app.add_server_event::<GameEvent>();
        app.register_type::<RestartCountdown>();
        app.register_type::<PostGameUiRoot>();
        app.add_systems((load_state,).in_schedule(OnEnter(GameState::Loading)));
        app.add_system(
            end_game_last_man_standing
                .run_if(is_server())
                .in_set(OnUpdate(GameState::Playing)),
        );
        app.add_system(show_post_game_text);
        app.add_system(update_restart_countdown);
        app.add_systems(
            (build_level.run_if(is_server()),).in_schedule(OnEnter(GameState::Playing)),
        );
        app.add_system(despawn_everything.in_schedule(OnEnter(GameState::MainMenu)));
        app.add_systems(
            (
                despawn_everything.run_if(is_server()),
                reload_with_current_players.run_if(is_server()),
            )
                .chain()
                .in_schedule(OnExit(GameState::PostGame)),
        );

        app.add_system(
            return_to_pregame_if_no_clients
                .run_if(is_server())
                .in_set(OnUpdate(GameState::PostGame)),
        );

        app.add_systems((asteroid_spawn,));
    }
}

#[derive(Component, Reflect, Default)]
pub struct Persist;

#[derive(Component, Reflect, Default)]
pub struct RestartCountdown {
    restart_at_time: f64,
}

impl RestartCountdown {
    pub const DEFAULT_RESTART_DELAY_SECONDS: f64 = 5.0;
    pub fn new(time: &Time) -> Self {
        Self {
            restart_at_time: time.elapsed_seconds_f64() + Self::DEFAULT_RESTART_DELAY_SECONDS,
        }
    }
}

pub fn load_state(mut next_state: ResMut<NextState<GameState>>) {
    info!("Auto-Advancing To Main Menu State");
    next_state.set(GameState::MainMenu);
}

///Should only be run by the server, and then fill backfill on the clients
pub fn build_level(mut cmds: Commands, time: Res<Time>) {
    let arena_size = Vec2::new(1800.0, 900.0);
    cmds.spawn(Arena {
        starting_size: arena_size,
        current_size: arena_size,
        time_spawned: time.elapsed_seconds(),
        friendly_force: Force::None,
    })
    .insert(Name::new("Arena"))
    .insert(Replication::default());

    let mut rng = rand::thread_rng();

    for _ in 0..rng.gen_range(1..5) {
        let x = rng.gen_range(-850.0..850.0);
        let y = rng.gen_range(-400.0..400.0);

        //Current these aren't really explicitly checked, intent was to make them composable.
        let powerup_type = match rng.gen_range(0..2) {
            0 => (PowerUp::RapidFire, Debuff::Slowed),
            1 => (PowerUp::Scattergun, Debuff::Inaccuracy),
            _ => (PowerUp::Scattergun, Debuff::Inaccuracy), //TODO: Rethink this, currently it's hardcoded and this isn't really used to compose
        };
        spawn_powerup(
            &mut cmds,
            Transform::from_xyz(x, y, 0.2),
            powerup_type.0,
            powerup_type.1,
        );
    }

    for _ in 0..rng.gen_range(30..45) {
        let roid_path = match (rng.gen_range(0..10) % 2) == 0 {
            true => RoidPath::One,
            false => RoidPath::Two,
        };

        let x = rng.gen_range(-850.0..850.0);
        let y = rng.gen_range(-400.0..400.0);
        let scale = rng.gen_range(16.0..64.0);
        let rotation = rng.gen_range(0.0..(PI * 2.0));

        let mut transform = Transform::from_xyz(x, y, 0.2);
        transform.rotate_z(rotation);
        cmds.spawn((
            Asteroid {
                scale,
                path: roid_path,
            },
            transform,
            Replication::default(),
        ));
    }
}

pub fn end_game_last_man_standing(
    query: Query<&Player, With<Player>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut game_events: EventWriter<ToClients<GameEvent>>,
) {
    if query.iter().count() <= 1 {
        if let Ok(player) = query.get_single() {
            game_events.send(ToClients {
                mode: SendMode::Broadcast,
                event: GameEvent::RoundWon {
                    winner: player.color,
                },
            });
        } else {
            game_events.send(ToClients {
                mode: SendMode::Broadcast,
                event: GameEvent::Tie,
            });
        }

        info!("Last Player Standing!");
        game_state.set(GameState::PostGame);
    }
}

#[derive(Component, Debug, Default, Reflect)]
pub struct PostGameUiRoot;

fn show_post_game_text(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut game_events: EventReader<GameEvent>,
    time: Res<Time>,
    ui_root_query: Query<Entity, With<PostGameUiRoot>>,
) {
    for event in game_events.iter() {
        info!("GameEvent from server {event:?}");
        let win_text = match event {
            GameEvent::RoundWon { winner } => (format!("{} wins!\n\n", winner), winner.color()),
            GameEvent::Tie => ("Tie!".to_string(), Color::YELLOW),
            GameEvent::Restart => {
                ui_root_query.for_each(|e| {
                    commands.entity(e).despawn_recursive();
                });
                return;
            }
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        size: Size {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                        },
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        gap: Size {
                            width: Default::default(),
                            height: Val::Px(24.0),
                        },
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                PostGameUiRoot,
            ))
            .with_children(|child_builder| {
                child_builder.spawn(TextBundle {
                    text: Text::from_section(
                        win_text.0,
                        TextStyle {
                            font: asset_server.load("hyperspace_font/Hyperspace Bold.otf"),
                            font_size: 36.0,
                            color: win_text.1,
                        },
                    ),
                    ..default()
                });
                child_builder.spawn((
                    RestartCountdown::new(time.as_ref()),
                    TextBundle {
                        text: Text::from_section(
                            format!(
                                "Restarting in {:.0}",
                                RestartCountdown::DEFAULT_RESTART_DELAY_SECONDS
                            ),
                            TextStyle {
                                font: asset_server.load("hyperspace_font/Hyperspace Bold.otf"),
                                font_size: 24.0,
                                color: Color::YELLOW,
                            },
                        ),
                        ..default()
                    },
                ));
            });
    }
}

fn update_restart_countdown(
    mut query: Query<(&mut Text, &RestartCountdown)>,
    time: Res<Time>,
    mut game_state: ResMut<NextState<GameState>>,
    server: Option<Res<RenetServer>>,
    mut game_events: EventWriter<ToClients<GameEvent>>,
) {
    for (mut text, countdown) in query.iter_mut() {
        let time_remaining = countdown.restart_at_time - time.elapsed_seconds_f64();
        text.sections[0].value = if time_remaining > 0.0 {
            format!("Restarting in {:.0}", time_remaining.ceil())
        } else {
            if server.is_some() {
                game_state.set(GameState::Playing);
                game_events.send(ToClients {
                    mode: SendMode::Broadcast,
                    event: GameEvent::Restart,
                })
            }
            "Restarting".to_string()
        };
    }
}

#[cfg(feature = "bevy_editor_pls")]
type PersistentRootEntities = (
    Without<Parent>,
    Without<Persist>,
    Without<Window>,
    Without<bevy_editor_pls::default_windows::hierarchy::HideInEditor>,
    Without<NoFrustumCulling>,
);
#[cfg(not(feature = "bevy_editor_pls"))]
type PersistentRootEntities = (
    Without<Parent>,
    Without<Window>,
    Without<Persist>,
    Without<NoFrustumCulling>,
);

fn despawn_everything(mut commands: Commands, query: Query<Entity, PersistentRootEntities>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn reload_with_current_players(
    mut commands: Commands,
    server: Res<RenetServer>,
    player_colors: Res<Players>,
) {
    commands.spawn_player(PlayerColor::Red, NetworkOwner(SERVER_ID));
    for client_id in server.clients_id() {
        if let Some(color) = player_colors.color(client_id) {
            commands.spawn_player(color, NetworkOwner(client_id));
        }
    }
}

fn return_to_pregame_if_no_clients(
    server: Res<RenetServer>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut removed: RemovedComponents<RestartCountdown>,
) {
    let countdown_finished = removed.iter().count() > 0;

    if countdown_finished && server.clients_id().is_empty() {
        warn!("No clients connected, returning to pregame");
        next_game_state.set(GameState::PreGame);
    }
}
