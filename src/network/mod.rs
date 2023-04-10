pub mod commands;
#[cfg(feature = "bevy_editor_pls")]
mod editor;
pub mod matchmaking;

use std::fmt::Debug;
use std::net::{IpAddr, Ipv4Addr};

use crate::asteroid::Asteroid;
use crate::bundles::lyon_rendering::roid_paths::RoidPath;
use crate::game_manager::GameState;
use crate::player::{Player, PlayerAction};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_rapier2d::prelude::Velocity;
use bevy_replicon::prelude::*;
use bevy_replicon::renet::ServerEvent;
use bevy_replicon::ReplicationPlugins;
use futures_lite::future;
use leafwing_input_manager::action_state::{ActionDiff, ActionState};
use leafwing_input_manager::systems::generate_action_diffs;
use leafwing_input_manager::Actionlike;
use serde::{Deserialize, Serialize};

use self::matchmaking::MatchmakingPlugin;

pub const DEFAULT_PORT: u16 = 4761;
pub const PROTOCOL_ID: u64 = 0;
pub const MAX_CLIENTS: usize = 6;
pub const MAX_MESSAGE_SIZE: u64 = 40000;

#[derive(Resource)]
pub struct NetworkInfo {
    pub public_ip: Option<IpAddr>,
    task: Task<Option<IpAddr>>,
}

#[derive(Copy, Clone, Debug)]
pub struct PublicIpFound(IpAddr);

fn poll_public_ip_task(mut network_info: ResMut<NetworkInfo>) {
    if network_info.public_ip.is_none() {
        let result: Option<Option<IpAddr>> =
            future::block_on(future::poll_once(&mut network_info.task));
        network_info.public_ip = result.flatten();
    }
}

impl Default for NetworkInfo {
    fn default() -> Self {
        Self {
            public_ip: None,
            task: Self::fetch_ip(),
        }
    }
}
impl NetworkInfo {
    pub fn fetch_ip() -> Task<Option<IpAddr>> {
        let thread_pool = AsyncComputeTaskPool::get();
        thread_pool.spawn(async move {
            match surf::get("https://api.ipify.org/").await {
                Ok(mut response) => match response.body_string().await {
                    Ok(ip_text) => match ip_text.parse::<Ipv4Addr>() {
                        Ok(address) => {
                            info!("Found Public IP: {}", address);
                            Some(IpAddr::V4(address))
                        }
                        Err(e) => {
                            warn!("Could not parse ip [{}]: {}", ip_text, e);
                            None
                        }
                    },
                    Err(e) => {
                        warn!("Could not marshal response body to string: {}", e);
                        None
                    }
                },
                Err(e) => {
                    warn!("Could not get get ip: {}", e);
                    None
                }
            }
        })
    }
}
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkInfo>();
        app.add_plugins(
            ReplicationPlugins
                .build()
                .set(ServerPlugin { tick_rate: 30 }),
        );
        app.add_plugin(MatchmakingPlugin);
        app.register_type::<NetworkOwner>();
        app.register_type::<RoidPath>();
        app.replicate::<Transform>();
        app.replicate::<Player>();
        app.replicate::<NetworkOwner>();
        app.replicate::<Asteroid>();
        app.replicate::<Velocity>();
        app.add_system(log_network_events);
        app.add_system(
            advance_to_playing_on_connected_and_replication
                .run_if(is_client())
                .in_set(OnUpdate(GameState::PreGame)),
        );
        app.add_client_event::<ActionDiff<PlayerAction, NetworkOwner>>();
        app.add_system(
            generate_action_diffs::<PlayerAction, NetworkOwner>.in_base_set(CoreSet::PostUpdate),
        );
        app.add_system(poll_public_ip_task);
        app.add_system(
            process_action_diffs::<PlayerAction, NetworkOwner>.in_base_set(CoreSet::PreUpdate),
        );

        #[cfg(feature = "bevy_editor_pls")]
        app.add_plugin(editor::EditorExtensionPlugin);
    }
}

pub fn is_server() -> impl FnMut(Option<Res<RenetServer>>) -> bool + Clone {
    resource_exists::<RenetServer>()
}

pub fn is_client() -> impl FnMut(Option<Res<RenetClient>>) -> bool + Clone {
    move |res: Option<Res<RenetClient>>| res.is_some()
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
            debug!("{:?}", action_diff);
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

fn advance_to_playing_on_connected_and_replication(
    client: Res<RenetClient>,
    mut next_game_state: ResMut<NextState<GameState>>,
    query: Query<With<Replication>>,
) {
    if client.is_connected() && query.iter().count() > 0 {
        next_game_state.set(GameState::Playing);
    }
}
