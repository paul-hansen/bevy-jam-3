// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
mod cli;

use crate::cli::CliPlugin;
use crate::game_manager::{GameState, Persist};
use crate::health::HealthPlugin;
use crate::network::NetworkPlugin;
use crate::player::PlayerPlugin;
use arena::ArenaPlugin;
use audio::SqueezeAudioPlugin;
use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomSettings};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_egui::EguiPlugin;
use bevy_mod_reqwest::ReqwestPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;
use bevy_rapier2d::prelude::{DebugRenderContext, NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::render::RapierDebugRenderPlugin;
use game_manager::GameManager;

mod arena;
mod asteroid;
mod audio;
mod bundles;
mod constructed_geometry;
mod forms;
mod game_manager;
mod health;
mod network;
mod player;
mod ui;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    #[cfg(feature = "bevy_editor_pls")]
    {
        use bevy_editor_pls::EditorPlugin;
        app.add_plugin(EditorPlugin::default());
    }

    app.add_state::<GameState>()
        .add_plugin(NetworkPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(ReqwestPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin {
            enabled: false,
            ..default()
        })
        .add_plugin(GameManager)
        .add_plugin(ArenaPlugin)
        .add_plugin(CliPlugin)
        .add_plugin(HealthPlugin)
        .add_plugin(bevy_kira_audio::AudioPlugin)
        .add_plugin(SqueezeAudioPlugin)
        .add_plugin(ui::UiPlugin)
        .insert_resource(Msaa::Sample8);

    app.register_type::<MainCamera>();

    if !app.is_plugin_added::<EguiPlugin>() {
        app.add_plugin(EguiPlugin);
    }

    app.add_startup_system(setup);
    app.add_system(debug_rapier);
    app.run();
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct MainCamera;

fn setup(mut commands: Commands) {
    commands.spawn((
        Persist,
        MainCamera,
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
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::AutoMin {
                    min_width: 1920.0,
                    min_height: 1080.0,
                },
                ..default()
            },
            ..default()
        },
        UiCameraConfig { show_ui: false },
        BloomSettings {
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        },
    ));
}

fn debug_rapier(mut debug_context: ResMut<DebugRenderContext>, keycodes: Res<Input<KeyCode>>) {
    if keycodes.just_released(KeyCode::F7) {
        debug_context.enabled = !debug_context.enabled;
    }
}
