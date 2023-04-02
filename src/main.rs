// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
mod cli;

use crate::cli::CliPlugin;
use crate::network::NetworkPlugin;
use crate::player::PlayerPlugin;
use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomSettings};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::ShapePlugin;
use bevy_rapier2d::prelude::{RapierPhysicsPlugin, NoUserData};
use bundles::lyon_rendering::TestRenderingPlugin;

mod bundles;
mod network;
mod player;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    #[cfg(feature = "bevy_editor_pls")]
    {
        use bevy_editor_pls::EditorPlugin;
        app.add_plugin(EditorPlugin::default());
    }

    app.add_plugin(CliPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(NetworkPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(TestRenderingPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .insert_resource(Msaa::Sample8)
        .add_startup_system(setup);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::rgb_u8(0, 0, 0)),
            },
            tonemapping: Tonemapping::TonyMcMapface,
            deband_dither: DebandDither::Enabled,
            ..default()
        },
        BloomSettings {
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        },
    ));
}
