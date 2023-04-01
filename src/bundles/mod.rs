use bevy::prelude::*;

use self::player_ship::PlayerShipBundle;

pub mod player_ship;

pub struct TestRenderingPlugin;

impl Plugin for TestRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_ship_system);
    }
}

pub fn spawn_ship_system(mut commands: Commands){
    info!("Spawning Ship");
    commands.spawn(PlayerShipBundle::default());
}