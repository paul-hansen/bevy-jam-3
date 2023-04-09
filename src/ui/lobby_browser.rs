use crate::ui::MenuUiContainer;
use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
pub struct LobbyBrowser {}

pub fn setup_lobby_browser(mut commands: Commands, menu_ui: Query<Entity, With<MenuUiContainer>>) {
    let menu_container = menu_ui.single();
    let entity = commands
        .spawn((
            Name::new("LobbyBrowser"),
            LobbyBrowser::default(),
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
