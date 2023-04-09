use crate::game_manager::GameState;
use crate::player::Player;
use crate::ui::Menu;
use bevy::{prelude::*, utils::HashMap};
use bevy_mod_reqwest::{ReqwestBytesResult, ReqwestClient, ReqwestRequest};
use bevy_replicon::server::ServerSet;
use serde::{Deserialize, Serialize};

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct MatchmakingState {
    pub lobby: Option<EphemeralMatchmakingLobby>,
    pub timer: Timer,
}

impl Default for MatchmakingState {
    fn default() -> Self {
        Self {
            lobby: None,
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
        }
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ServerList {
    pub servers: HashMap<String, EphemeralMatchmakingLobby>,
}

#[derive(Component)]
pub struct PostLobbyReq;

#[derive(Component)]
pub struct GetLobbyReq;

#[derive(Debug, Serialize, Deserialize, Reflect, FromReflect)]
#[serde(rename_all = "camelCase")]
pub struct EphemeralMatchmakingLobby {
    pub ip: String,
    pub name: String,
    #[serde(alias = "playerCapacity")]
    pub player_capacity: u8,
    #[serde(alias = "slotsOccupied")]
    pub slots_occupied: u8,
    #[serde(alias = "autoRestart")]
    pub auto_restart: bool,
    #[serde(alias = "hasPassword")]
    pub has_password: bool,
    #[serde(alias = "lastUpdated")]
    pub last_updated: u64,
}

pub fn update_matchmaking_state(
    mut mm_res: ResMut<MatchmakingState>,
    mut cmds: Commands,
    time: Res<Time>,
    client: ResMut<ReqwestClient>,
    menu_state: Res<State<Menu>>,
) {
    mm_res.timer.tick(time.delta());

    if mm_res.timer.just_finished() {
        let url = "http://matchmaking.braymatter.com:8091/api/v1/matchmaking/ephemeral/lobbies";

        if let Some(hosted_lobby) = &mm_res.lobby {
            let Ok(json) = serde_json::to_string(&hosted_lobby) else {
                warn!("Could not serialize ephemeral lobby to json");
                return;
            };

            if let Ok(postreq) = client
                .0
                .post(url)
                .body(json.clone())
                .header("Content-Type", "Application/JSON")
                .build()
            {
                info!("Sending req: {}", json);
                let req = ReqwestRequest(Some(postreq));
                cmds.spawn(req).insert(PostLobbyReq);
            } else {
                warn!("Could not construct request");
            };
        }

        if menu_state.0 == Menu::LobbyBrowser {
            if let Ok(getreq) = client.0.get(url).build() {
                cmds.spawn(ReqwestRequest(Some(getreq))).insert(GetLobbyReq);
            } else {
                warn!("Could not construct request to pull serverlist");
            }
        }
    }
}

pub fn consume_matchmaking_responses(
    get_responses: Query<(&ReqwestBytesResult, Entity), With<GetLobbyReq>>,
    post_responses: Query<(&ReqwestBytesResult, Entity), With<PostLobbyReq>>,
    mut cmds: Commands,
    mut server_list: ResMut<ServerList>,
) {
    get_responses.iter().for_each(|(response, ent)| {
        if let Some(response) = response.as_str() {
            match serde_json::from_str::<HashMap<String, EphemeralMatchmakingLobby>>(response) {
                Ok(res) => {
                    server_list.servers = res;
                }
                Err(e) => {
                    warn!(
                        "Got error when deserializing server list: {} \n {}",
                        e, response
                    );
                }
            }
        } else {
            warn!("No response fetching server list");
        };

        cmds.entity(ent).despawn_recursive();
    });

    post_responses.iter().for_each(|(response, ent)| {
        if let Some(response) = response.as_str() {
            match response {
                "SUCCESS" => {
                    info!("Successfully Posted MM Lobby")
                }
                anything_else => {
                    info!("NON SUCCESS: {}", anything_else)
                }
            }
        } else {
            warn!("No response submitting lobby to master server. Lobby will not be visible.");
        };

        cmds.entity(ent).despawn_recursive();
    });
}

fn remove_lobby_info(mut matchmaking_state: ResMut<MatchmakingState>) {
    matchmaking_state.lobby = None;
}

fn update_lobby_info(mut matchmaking_state: ResMut<MatchmakingState>, query: Query<With<Player>>) {
    let player_count = query.iter().count();
    if player_count
        != matchmaking_state
            .lobby
            .as_ref()
            .map(|x| x.player_capacity as usize)
            .unwrap_or_default()
    {
        if let Some(lobby) = matchmaking_state.lobby.as_mut() {
            lobby.slots_occupied = player_count as u8;
        }
    }
}

pub struct MatchmakingPlugin;

impl Plugin for MatchmakingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MatchmakingState>();
        app.register_type::<ServerList>();
        app.register_type::<HashMap<String, EphemeralMatchmakingLobby>>();
        app.init_resource::<MatchmakingState>();
        app.init_resource::<ServerList>();
        app.add_system(update_matchmaking_state);
        app.add_system(consume_matchmaking_responses);

        // Remove lobby info so it doesn't keep notifying the master server.
        app.add_system(remove_lobby_info.in_schedule(OnEnter(GameState::MainMenu)));
        app.add_system(update_lobby_info.in_set(ServerSet::Authority));
    }
}
