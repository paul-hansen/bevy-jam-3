mod util;

use std::net::IpAddr;

use crate::player::Player;
use bevy::prelude::*;
use bevy_replicon::prelude::AppReplicationExt;
use bevy_replicon::renet::ServerEvent;
use bevy_replicon::ReplicationPlugins;

#[derive(Resource, Eq, PartialEq, PartialOrd, Ord)]
struct NetworkInfo {
    pub public_ip: IpAddr,
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ReplicationPlugins);
        app.replicate::<Transform>();
        app.replicate::<Player>();
        app.add_system(log_network_events);
    }
}

fn log_network_events(mut events: EventReader<ServerEvent>) {
    for event in events.iter() {
        info!("{event:?}");
    }
}
