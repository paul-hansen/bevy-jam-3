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
use bevy_prototype_lyon::prelude::ShapePlugin;
use bundles::TestRenderingPlugin;

mod bundles;
mod network;
mod player;
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(CliPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(NetworkPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(TestRenderingPlugin)
        .add_startup_system(setup);

    #[cfg(feature = "bevy_editor_pls")]
    {
        use bevy_editor_pls::EditorPlugin;
        app.add_plugin(EditorPlugin::default());
    }

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
