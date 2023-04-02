#[cfg(feature = "bevy_editor_pls")]
mod editor;
mod util;

use std::fmt::Debug;
use std::net::{IpAddr, Ipv4Addr};

use crate::asteroid::Asteroid;
use crate::player::{Player, PlayerAction};
use bevy::prelude::*;
use bevy::tasks::{IoTaskPool, TaskPool};
use bevy_replicon::prelude::{AppReplicationExt, ClientEventAppExt, FromClient};
use bevy_replicon::renet::ServerEvent;
use bevy_replicon::ReplicationPlugins;
use leafwing_input_manager::action_state::{ActionDiff, ActionState};
use leafwing_input_manager::systems::generate_action_diffs;
use leafwing_input_manager::Actionlike;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct NetworkInfo {
    pub public_ip: Option<IpAddr>,
    task_pool: &'static IoTaskPool,
}

impl Default for NetworkInfo {
    fn default() -> Self {
        let mut val = Self {
            public_ip: Default::default(),
            task_pool: IoTaskPool::init(TaskPool::new),
        };

        val.fetch_ip();

        val
    }
}
impl NetworkInfo {
    pub fn fetch_ip(&mut self) {
        self.task_pool.scope(|scope| {
            scope.spawn(async {
                match surf::get("https://api.ipify.org/").await {
                    Ok(mut response) => match response.body_string().await {
                        Ok(ip_text) => match ip_text.parse::<Ipv4Addr>() {
                            Ok(address) => {
                                info!("Found Public IP: {}", address);
                                self.public_ip = Some(IpAddr::V4(address));
                            }
                            Err(e) => {
                                self.public_ip = None;
                                warn!("Could not parse ip [{}]: {}", ip_text, e)
                            }
                        },
                        Err(e) => {
                            self.public_ip = None;
                            warn!("Could not marshal response body to string: {}", e)
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
        app.register_type::<NetworkOwner>();
        app.replicate::<Transform>();
        app.replicate::<Player>();
        app.replicate::<NetworkOwner>();
        app.replicate::<Asteroid>();
        app.add_system(log_network_events);
        app.add_client_event::<ActionDiff<PlayerAction, NetworkOwner>>();
        app.add_system(
            generate_action_diffs::<PlayerAction, NetworkOwner>.in_base_set(CoreSet::PostUpdate),
        );
        app.add_system(
            process_action_diffs::<PlayerAction, NetworkOwner>.in_base_set(CoreSet::PreUpdate),
        );

        #[cfg(feature = "bevy_editor_pls")]
        app.add_plugin(editor::RenetEditorWindow);
    }
}

/// This is the same as [`leafwing_input_manager::systems::process_action_diffs`] but
/// uses [`FromClient`] to read the events
pub fn process_action_diffs<A: Actionlike + Debug, ID: Eq + Component + Clone + Debug>(
    mut action_state_query: Query<(&mut ActionState<A>, &ID)>,
    mut action_diffs: EventReader<FromClient<ActionDiff<A, ID>>>,
) {
    // PERF: This would probably be faster with an index, but is much more fussy
    for action_diff in action_diffs.iter() {
        let action_diff = &action_diff.event;
        for (mut action_state, id) in action_state_query.iter_mut() {
            info!("{:?}", action_diff);
            match action_diff {
                ActionDiff::Pressed {
                    action,
                    id: event_id,
                } => {
                    if event_id == id {
                        action_state.press(action.clone());
                        continue;
                    }
                }
                ActionDiff::Released {
                    action,
                    id: event_id,
                } => {
                    if event_id == id {
                        action_state.release(action.clone());
                        continue;
                    }
                }
            };
        }
    }
}

/// Which client id owns this entity?
#[derive(Component, Reflect, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct NetworkOwner(pub u64);

impl Default for NetworkOwner {
    fn default() -> Self {
        Self(u64::MAX)
    }
}

fn log_network_events(mut events: EventReader<ServerEvent>) {
    for event in events.iter() {
        info!("{event:?}");
    }
}
