use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_replicon::prelude::{SendMode, ServerEventAppExt, ToClients};
use bevy_replicon::replication_core::Replication;
use egui::Align2;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::forms::{ConnectForm, ListenForm};

use crate::network::{is_server, NetworkInfo};
use crate::player::{Player, PlayerColor};
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
}

pub struct GameManager;

impl Plugin for GameManager {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>();
        app.add_server_event::<GameEvent>();
        app.add_systems((load_state,).in_schedule(OnEnter(GameState::Loading)));
        app.add_systems((draw_main_menu,).in_set(OnUpdate(GameState::MainMenu)));
        app.add_system(
            end_game_last_man_standing
                .run_if(is_server())
                .in_set(OnUpdate(GameState::Playing)),
        );
        app.add_system(show_post_game_text);
        app.add_systems(
            (build_level.run_if(is_server()),).in_schedule(OnEnter(GameState::Playing)),
        );

        app.add_systems((asteroid_spawn,));
    }
}

pub fn load_state(mut next_state: ResMut<NextState<GameState>>) {
    info!("Auto-Advancing To Main Menu State");
    next_state.set(GameState::MainMenu);
}

pub fn draw_main_menu(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut connect_form: Local<ConnectForm>,
    mut listen_form: Local<ListenForm>,
    network_info: Res<NetworkInfo>,
) {
    if network_info.is_changed() {
        if let Some(ip) = network_info.public_ip {
            listen_form.ip = ip.to_string();
        }
    }
    egui::Window::new("Main Menu")
        .auto_sized()
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Join Game");
            connect_form.draw(ui);
            if ui.button("Join").clicked() {
                if let Ok(connect) = connect_form.validate() {
                    commands.add(connect);
                }
            }
            ui.add_space(14.0);
            ui.push_id("host", |ui| {
                ui.heading("Host Game");
                listen_form.draw(ui);
                if ui.button("Host").clicked() {
                    if let Ok(listen) = listen_form.validate() {
                        commands.add(listen);
                    }
                }
            });
        });
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
    for _ in 0..5 {
        let roid_path = match (rng.gen_range(0..10) % 2) == 0 {
            true => RoidPath::One,
            false => RoidPath::Two,
        };

        let x = rng.gen_range(-300.0..300.0);
        let y = rng.gen_range(-300.0..300.0);
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

fn show_post_game_text(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut game_events: EventReader<GameEvent>,
) {
    for event in game_events.iter() {
        info!("GameEvent from server {event:?}");
        let win_text = match event {
            GameEvent::RoundWon { winner } => (format!("{} wins!\n\n", winner), winner.color()),
            GameEvent::Tie => ("Tie!".to_string(), Color::YELLOW),
        };

        commands
            .spawn(NodeBundle {
                style: Style {
                    size: Size {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                    },
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|child_builder| {
                child_builder.spawn(TextBundle {
                    text: Text::from_sections(vec![
                        TextSection::new(
                            win_text.0,
                            TextStyle {
                                font: asset_server.load("VectorBattleFont/Vectorb.ttf"),
                                font_size: 36.0,
                                color: win_text.1,
                            },
                        ),
                        // TextSection::new(
                        //     "Press enter to play again",
                        //     TextStyle {
                        //         font: asset_server.load("VectorBattleFont/Vectorb.ttf"),
                        //         font_size: 24.0,
                        //         color: Color::YELLOW,
                        //     },
                        // ),
                    ])
                    .with_alignment(TextAlignment::Center),
                    ..default()
                });
            });
    }
}
