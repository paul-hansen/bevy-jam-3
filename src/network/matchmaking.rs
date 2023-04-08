use bevy::{prelude::*, utils::HashMap};
use bevy_mod_reqwest::{
    ReqwestClient, ReqwestRequest, ReqwestBytesResult,
};
use serde::{Deserialize, Serialize};

#[derive(Resource, Default)]
pub struct MatchmakingState {
    pub lobby: Option<EphemeralMatchmakingLobby>,
    pub server_list: HashMap<String, EphemeralMatchmakingLobby>,
    pub timer: Timer,
}

#[derive(Component)]
pub struct PostLobbyReq;

#[derive(Component)]
pub struct GetLobbyReq;


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct EphemeralMatchmakingLobby {
    pub ip: String,
    pub name: String,
    #[serde(alias = "playerCapacity")]
    pub player_capacity: u8,
    #[serde(alias = "slotsOccupied")]
    pub slots_occupied: u8,
    #[serde(alias = "autoRestart")]
    pub auto_resart: bool,
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
) {
    mm_res.timer.tick(time.delta());

    if mm_res.timer.just_finished() {
      let url = "http://localhost:8091/api/v1/matchmaking/ephemeral/lobbies";

      if let Some(hosted_lobby) = &mm_res.lobby {
        let Ok(json) = serde_json::to_string(&hosted_lobby) else{
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

      if let Ok(getreq) = client.0.get(url).build() {
        cmds.spawn(ReqwestRequest(Some(getreq))).insert(GetLobbyReq);
      }else{
        warn!("Could not construct request to pull serverlist");
      }
      
    }
}

pub fn consume_matchmaking_responses(get_responses: Query<(&ReqwestBytesResult, Entity), With<GetLobbyReq>>, post_responses: Query<(&ReqwestBytesResult, Entity), With<PostLobbyReq>>, mut cmds: Commands, mut mm_res: ResMut<MatchmakingState>){
  get_responses.iter().for_each(|(response, ent)|{
    match serde_json::from_str::<HashMap<String, EphemeralMatchmakingLobby>>(response.as_str().unwrap()){
      Ok(res) => {
        mm_res.server_list = res;
      },
      Err(e) => {
        warn!("Got error when deserializing server list: {} \n {}", e, response.as_str().unwrap());
      }
    }

    cmds.entity(ent).despawn_recursive();
  });

  post_responses.iter().for_each(|(response, ent)| {
    match response.as_str().unwrap(){
      "SUCCESS" => {info!("Successfully Posted MM Lobby")},
      anything_else => {info!("NON SUCCESS: {}", anything_else)}
    }

    cmds.entity(ent).despawn_recursive();
  });
}

pub fn initialize_matchmaking_poller(mut mm_res: ResMut<MatchmakingState>) {
    info!("Initializing Matchmaking Poller");
    mm_res.timer = Timer::from_seconds(3.0, TimerMode::Repeating);
}

//TODO: This is a test fn. 
pub fn register_server_host(mut mm_res: ResMut<MatchmakingState>){
  mm_res.lobby = Some(EphemeralMatchmakingLobby { ip: "ME".to_string(), name: "MY GAME".to_string(), player_capacity: 5_u8, slots_occupied: 1, auto_resart: true, has_password: false, last_updated: 0 })
}
pub struct MatchmakingPlugin;

impl Plugin for MatchmakingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MatchmakingState>();
        app.add_startup_systems((initialize_matchmaking_poller, register_server_host));
        app.add_system(update_matchmaking_state);
        app.add_system(consume_matchmaking_responses);
    }
}