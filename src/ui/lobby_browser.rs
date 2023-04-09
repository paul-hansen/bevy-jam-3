use crate::network::commands::NetworkCommandsExt;
use crate::network::matchmaking::ServerList;
use crate::network::DEFAULT_PORT;
use crate::ui::{change_button_text_color, Menu, MenuUiContainer};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
pub struct LobbyListContainer;

pub fn setup_lobby_browser(
    mut commands: Commands,
    menu_ui: Query<Entity, With<MenuUiContainer>>,
    asset_server: ResMut<AssetServer>,
) {
    let font = asset_server.load("hyperspace_font/Hyperspace Bold.otf");
    let menu_container = menu_ui.single();
    let entity = commands
        .spawn((
            Name::new("LobbyBrowser"),
            Menu::LobbyBrowser,
            NodeBundle {
                style: Style {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|child_builder| {
            child_builder.spawn(TextBundle {
                text: Text::from_section(
                    "Lobby Browser",
                    TextStyle {
                        font: font.clone(),
                        font_size: 32.0,
                        color: Color::YELLOW,
                    },
                ),
                style: Style {
                    margin: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                ..default()
            });
            child_builder.spawn((
                LobbyListContainer,
                NodeBundle {
                    background_color: BackgroundColor::from(Color::YELLOW),
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                },
            ));
            child_builder
                .spawn(ButtonBundle {
                    background_color: BackgroundColor::from(Color::BLACK),
                    ..default()
                })
                .with_children(|cb| {
                    cb.spawn(TextBundle {
                        text: Text::from_section(
                            "Join by IP",
                            TextStyle {
                                font: font.clone(),
                                font_size: 32.0,
                                color: Color::YELLOW,
                            },
                        ),
                        ..default()
                    });
                });
        })
        .id();
    commands.entity(menu_container).add_child(entity);
}

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
pub struct JoinGameButton {
    pub ip: String,
}

pub fn handle_join_game_click(
    mut commands: Commands,
    mut query: Query<(Entity, &Interaction, &JoinGameButton), Changed<Interaction>>,
    children: Query<&Children>,
    mut texts: Query<&mut Text>,
) {
    for (entity, interaction, join_game) in query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                warn!("clicked");
                if let Ok(ip) = IpAddr::from_str(&join_game.ip) {
                    commands.connect(ip, Ipv4Addr::new(0, 0, 0, 0).into(), DEFAULT_PORT);
                };
            }
            Interaction::Hovered => {
                change_button_text_color(entity, &children, &mut texts, Color::ORANGE);
            }
            Interaction::None => {
                change_button_text_color(entity, &children, &mut texts, Color::RED);
            }
        }
    }
}

pub fn update_lobby_browser(
    mut commands: Commands,
    server_list: Res<ServerList>,
    query: Query<Entity, With<LobbyListContainer>>,
    asset_server: ResMut<AssetServer>,
) {
    let Ok(container) = query.get_single() else {
        return
    };
    commands.entity(container).despawn_descendants();
    let font = asset_server.load("hyperspace_font/Hyperspace Bold.otf");
    commands.entity(container).with_children(|child_builder| {
        // Header row
        row_builder(
            child_builder,
            font.clone(),
            vec![
                (100.0, "Name".to_string()),
                (100.0, "Players".to_string()),
                (100.0, "Password".to_string()),
                (100.0, "".to_string()),
            ],
        );
        for lobby in server_list.servers.values() {
            row_builder(
                child_builder,
                font.clone(),
                vec![
                    (100.0, lobby.name.to_string()),
                    (
                        100.0,
                        format!("{} / {}", lobby.slots_occupied, lobby.player_capacity),
                    ),
                    (100.0, lobby.has_password.to_string()),
                ],
            )
            .with_children(|child_builder| {
                child_builder
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                margin: UiRect::vertical(Val::Px(1.0)),
                                size: Size::width(Val::Px(100.0)),
                                ..default()
                            },
                            background_color: BackgroundColor::from(Color::BLACK),
                            ..default()
                        },
                        JoinGameButton {
                            ip: lobby.ip.clone(),
                        },
                    ))
                    .with_children(|cb| {
                        cb.spawn(TextBundle {
                            text: Text::from_section(
                                "Join",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 24.0,
                                    color: Color::RED,
                                },
                            ),
                            ..default()
                        });
                    });
            });
        }
    });
}

fn row_builder<'w, 's, 'a>(
    child_builder: &'a mut ChildBuilder<'w, 's, '_>,
    font: Handle<Font>,
    columns: Vec<(f32, String)>,
) -> EntityCommands<'w, 's, 'a> {
    let mut row = child_builder.spawn(NodeBundle {
        background_color: BackgroundColor::from(Color::NONE),
        style: Style {
            flex_direction: FlexDirection::Row,

            ..default()
        },
        ..default()
    });
    for column in columns {
        row.with_children(|child_builder| {
            child_builder.spawn(TextBundle {
                text: Text::from_section(
                    column.1,
                    TextStyle {
                        font: font.clone(),
                        font_size: 24.0,
                        color: Color::YELLOW,
                    },
                ),
                style: Style {
                    margin: UiRect::vertical(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(0.0)),
                    size: Size::width(Val::Px(column.0)),
                    ..default()
                },
                background_color: BackgroundColor::from(Color::BLACK),
                ..default()
            });
        });
    }
    row
}
