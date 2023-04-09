use crate::network::commands::NetworkCommandsExt;
use crate::ui::{MenuState, MenuUiContainer};
use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ConfirmQuit;

pub fn setup_confirm_quit(
    mut commands: Commands,
    menu_ui: Query<Entity, With<MenuUiContainer>>,
    asset_server: ResMut<AssetServer>,
) {
    let menu_container = menu_ui.single();
    let entity = commands
        .spawn((
            Name::new("ConfirmQuit"),
            ConfirmQuit::default(),
            NodeBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(10.0)),
                    display: Display::None,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|child_builder| {
            child_builder.spawn(TextBundle::from_section(
                "Quit to menu? Y/N",
                TextStyle {
                    font: asset_server.load("hyperspace_font/Hyperspace Bold.otf"),
                    font_size: 24.0,
                    color: Color::YELLOW,
                },
            ));
        })
        .id();
    commands.entity(menu_container).add_child(entity);
}

pub fn confirm_quit_to_menu_update(
    mut commands: Commands,
    key_codes: Res<Input<KeyCode>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    if key_codes.just_released(KeyCode::Y) {
        commands.disconnect();
        next_menu_state.set(MenuState::Hidden);
    }
    if key_codes.just_released(KeyCode::N) {
        next_menu_state.set(MenuState::Hidden);
    }
}
