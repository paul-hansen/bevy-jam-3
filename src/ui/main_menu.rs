use crate::ui::MenuUiContainer;
use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MainMenu;

pub fn setup_main_menu(mut commands: Commands, menu_ui: Query<Entity, With<MenuUiContainer>>) {
    let menu_container = menu_ui.single();
    let entity = commands
        .spawn((
            Name::new("MainMenu"),
            MainMenu::default(),
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
