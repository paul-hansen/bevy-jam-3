use crate::network::commands::Disconnect;
use crate::ui::{CommandOnClick, Menu, MenuUiContainer};
use bevy::prelude::*;
use bevy_replicon::prelude::RenetServer;

#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]
pub struct PreGameText;

pub fn setup_pre_game(
    mut commands: Commands,
    menu_ui: Query<Entity, With<MenuUiContainer>>,
    asset_server: Res<AssetServer>,
) {
    let menu_container = menu_ui.single();
    let font = asset_server.load("hyperspace_font/Hyperspace Bold.otf");
    let entity = commands
        .spawn((
            Name::new("PreGame"),
            Menu::PreGame,
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
            cb.spawn((
                PreGameText,
                TextBundle {
                    text: Text::from_section(
                        "Waiting for Connection",
                        TextStyle {
                            font: font.clone(),
                            font_size: 48.0,
                            color: Color::YELLOW,
                        },
                    ),
                    style: Style {
                        margin: UiRect::bottom(Val::Px(24.0)),
                        ..default()
                    },
                    ..default()
                },
            ));

            cb.spawn((
                ButtonBundle {
                    background_color: BackgroundColor::from(Color::BLACK),
                    ..default()
                },
                CommandOnClick {
                    command: Disconnect,
                },
            ))
            .with_children(|cb| {
                cb.spawn(TextBundle {
                    text: Text::from_section(
                        "Leave",
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

pub fn update_pre_game_text(
    mut query: Query<&mut Text, With<PreGameText>>,
    server: Option<Res<RenetServer>>,
) {
    for mut text in query.iter_mut() {
        text.sections[0].value = if server.is_some() {
            "Waiting for Players".to_string()
        } else {
            "Connecting to Server".to_string()
        }
    }
}
