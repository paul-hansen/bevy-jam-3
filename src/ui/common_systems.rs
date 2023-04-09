use bevy::prelude::*;

pub fn set_display_flex<C: Component>(mut query: Query<&mut Style, With<C>>) {
    for mut style in query.iter_mut() {
        style.display = Display::Flex;
    }
}

pub fn set_display_none<C: Component>(mut query: Query<&mut Style, With<C>>) {
    for mut style in query.iter_mut() {
        style.display = Display::None;
    }
}
