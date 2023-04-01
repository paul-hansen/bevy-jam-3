// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod cli;

use crate::cli::CliPlugin;
use crate::network::NetworkPlugin;
use crate::player::PlayerPlugin;
use bevy::prelude::*;

mod network;
mod player;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(CliPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(NetworkPlugin)
        .add_startup_system(setup)
        .run();

    #[cfg(feature = "bevy_editor_pls")]
    {
        use bevy_editor_pls::EditorPlugin;
        app.add_plugin(EditorPlugin::default());
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
