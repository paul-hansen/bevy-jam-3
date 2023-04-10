use crate::ui::{ChangeStateOnClick, Menu, MenuUiContainer};
use bevy::prelude::*;

pub fn setup_main_menu(
    mut commands: Commands,
    menu_ui: Query<Entity, With<MenuUiContainer>>,
    asset_server: Res<AssetServer>,
) {
    let menu_container = menu_ui.single();
    let font = asset_server.load("hyperspace_font/Hyperspace Bold.otf");
    let entity = commands
        .spawn((
            Name::new("MainMenu"),
            Menu::Main,
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    display: Display::None,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|cb| {
            cb.spawn(TextBundle {
                text: Text::from_section(
                    "Stellar Squeezebox",
                    TextStyle {
                        font: font.clone(),
                        font_size: 52.0,
                        color: Color::YELLOW,
                    },
                ),
                style: Style {
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                },
                ..default()
            });

            cb.spawn((
                ButtonBundle {
                    background_color: BackgroundColor::from(Color::BLACK),
                    ..default()
                },
                ChangeStateOnClick {
                    state: Menu::CreateGame,
                },
            ))
            .with_children(|cb| {
                cb.spawn(TextBundle {
                    text: Text::from_section(
                        "Create Game",
                        TextStyle {
                            font: font.clone(),
                            font_size: 32.0,
                            color: Color::YELLOW,
                        },
                    ),
                    ..default()
                });
            });

            cb.spawn((
                ButtonBundle {
                    background_color: BackgroundColor::from(Color::BLACK),
                    ..default()
                },
                ChangeStateOnClick {
                    state: Menu::LobbyBrowser,
                },
            ))
            .with_children(|cb| {
                cb.spawn(TextBundle {
                    text: Text::from_section(
                        "Join Game",
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
