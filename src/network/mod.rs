mod util;

use std::net::{IpAddr, Ipv4Addr};

use bevy::{prelude::*, tasks::{IoTaskPool, TaskPool}};
use crate::player::Player;
use bevy::prelude::*;
use bevy_replicon::prelude::AppReplicationExt;
use bevy_replicon::renet::ServerEvent;
use bevy_replicon::ReplicationPlugins;
#[derive(Resource)]
pub struct NetworkInfo {
    pub public_ip: Option<IpAddr>,
    task_pool: &'static IoTaskPool,
}

impl Default for NetworkInfo{
    fn default() -> Self {
        let mut val = Self { public_ip: Default::default(), task_pool: IoTaskPool::init(||{TaskPool::new()}) };
        
        val.fetch_ip();
        
        val
    }
}
impl NetworkInfo {
    pub fn fetch_ip(&mut self) {
        self.task_pool.scope(|scope| {
            scope.spawn(async {
              match surf::get("https://api.ipify.org/").await{
                Ok(mut response) => {
                  match response.body_string().await{
                    Ok(ip_text) => {
                      match ip_text.parse::<Ipv4Addr>() {
                        Ok(address) => {
                          info!("Found Public IP: {}", address);
                          self.public_ip = Some(IpAddr::V4(address));
                        }, 
                        Err(e) => {
                          self.public_ip = None;
                          warn!("Could not parse ip [{}]: {}", ip_text, e)
                        }
                      }
                    },
                    Err(e) => {
                      self.public_ip = None;
                      warn!("Could not marshal response body to string: {}", e)
                    }
                  }
                }, 
                Err(e) => {
                  self.public_ip = None;
                  warn!("Could not get get ip: {}", e);
                }
              }
            });
        });
    }
}
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkInfo>();
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
