use bevy::prelude::*;
use bevy_replicon::renet::RenetServer;

#[derive(Debug, Hash, Eq, PartialEq, Clone, States, Default)]
pub enum GameState{
  #[default]
  Loading,
  MainMenu,
  PreGame,
  Playing,
  PostGame,
}

pub struct GameManager;

impl Plugin for GameManager {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>();
        app.add_systems((build_level,).in_schedule(OnEnter(GameState::Playing)));
    }
}

///Should only be run by the server, and then fill backfill on the clients
pub fn build_level(mut cmds: Commands, server_resource: Option<Res<RenetServer>>) {
    if server_resource.is_none() {
        return;
    }

    let level_root = cmds.spawn(Name::new("LevelRoot")).id();

    for i in 0..15 {
        
    }
}
