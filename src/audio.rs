use bevy::prelude::{Res, Plugin, AssetServer, App};
use bevy_kira_audio::{prelude::{Audio}, AudioControl};

pub struct SqueezeAudioPlugin;

pub fn load_and_start_music(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play(asset_server.load("stellar_squeezebox.mp3")).looped();
}

impl Plugin for SqueezeAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_and_start_music);
    }
}
