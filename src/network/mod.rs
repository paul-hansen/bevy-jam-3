mod util;

use std::net::IpAddr;

use bevy::prelude::*;

#[derive(Resource, Eq, PartialEq, PartialOrd, Ord)]
struct NetworkInfo{
  pub public_ip: IpAddr,
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin{
    fn build(&self, app: &mut App) {
        todo!()
    }
}