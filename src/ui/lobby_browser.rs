use crate::ui::{Menu, MenuUiContainer};
use bevy::prelude::*;

pub fn setup_lobby_browser(mut commands: Commands, menu_ui: Query<Entity, With<MenuUiContainer>>) {
    let menu_container = menu_ui.single();
    let entity = commands
        .spawn((
            Name::new("LobbyBrowser"),
            Menu::LobbyBrowser,
            NodeBundle {
                style: Style {
                    display: Display::None,
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    commands.entity(menu_container).add_child(entity);
}
