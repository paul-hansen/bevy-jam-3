use bevy::prelude::*;

pub struct AudioPlugin;

pub fn load_and_start_music(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play(asset_server.load("stellar_squeezebox.mp3"));
}

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_and_start_music);
    }
}
