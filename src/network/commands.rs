use crate::game_manager::GameState;
use crate::network::matchmaking::{EphemeralMatchmakingLobby, MatchmakingState};
use crate::network::{NetworkOwner, DEFAULT_PORT, MAX_CLIENTS, MAX_MESSAGE_SIZE, PROTOCOL_ID};
use crate::player::commands::SpawnPlayer;
use crate::player::{PlayerColor, Players};
use bevy::ecs::system::{Command, SystemState};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_replicon::prelude::*;
use bevy_replicon::renet::{
    ChannelConfig, ClientAuthentication, RenetConnectionConfig, ServerAuthentication, ServerConfig,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::time::SystemTime;

pub trait NetworkCommandsExt {
    fn connect(&mut self, ip: IpAddr, bind: IpAddr, port: u16);
    fn listen(&mut self, ip: IpAddr, bind: IpAddr, port: u16, server_name: String);
    fn disconnect(&mut self);
}

impl<'w, 's> NetworkCommandsExt for Commands<'w, 's> {
    fn connect(&mut self, ip: IpAddr, bind: IpAddr, port: u16) {
        self.add(Connect { bind, ip, port });
    }

    fn listen(&mut self, ip: IpAddr, bind: IpAddr, port: u16, server_name: String) {
        self.add(Listen {
            bind,
            port,
            ip,
            server_name,
        });
    }

    fn disconnect(&mut self) {
        self.add(Disconnect);
    }
}

#[derive(Clone, Debug)]
pub struct Connect {
    pub bind: IpAddr,
    pub ip: IpAddr,
    pub port: u16,
}

impl Default for Connect {
    fn default() -> Self {
        Self {
            bind: Ipv4Addr::new(0, 0, 0, 0).into(),
            ip: Ipv4Addr::new(127, 0, 0, 1).into(),
            port: DEFAULT_PORT,
        }
    }
}

impl Command for Connect {
    fn write(self, world: &mut World) {
        let client = {
            let mut state = SystemState::<(
                Res<NetworkChannels>,
                Query<&mut Window, With<PrimaryWindow>>,
            )>::new(world);
            let (network_channels, mut primary_window) = state.get_mut(world);
            let mut receive_channels_config = network_channels.server_channels();
            apply_message_size_to_channels(&mut receive_channels_config);
            let mut send_channels_config = network_channels.client_channels();
            apply_message_size_to_channels(&mut send_channels_config);
            let current_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            let client_id = current_time.as_millis() as u64;
            let server_addr = SocketAddr::new(self.ip, self.port);
            let socket = UdpSocket::bind((self.bind, 0)).expect("0.0.0.0 should be bindable");
            let authentication = ClientAuthentication::Unsecure {
                client_id,
                protocol_id: PROTOCOL_ID,
                server_addr,
                user_data: None,
            };

            let connection_config = RenetConnectionConfig {
                send_channels_config,
                receive_channels_config,
                ..Default::default()
            };
            primary_window.single_mut().title = "Client".to_string();

            RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
        };
        world
            .resource_mut::<NextState<GameState>>()
            .set(GameState::PreGame);
        world.insert_resource(client);
    }
}

pub struct Listen {
    pub bind: IpAddr,
    pub ip: IpAddr,
    pub port: u16,
    pub server_name: String,
}

impl Command for Listen {
    fn write(self, world: &mut World) {
        let server = {
            let mut state = SystemState::<(
                Res<NetworkChannels>,
                Query<&mut Window, With<PrimaryWindow>>,
            )>::new(world);
            let (network_channels, mut primary_window) = state.get_mut(world);
            let mut send_channels_config = network_channels.server_channels();
            apply_message_size_to_channels(&mut send_channels_config);
            let mut receive_channels_config = network_channels.client_channels();
            apply_message_size_to_channels(&mut receive_channels_config);

            let current_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            let server_addr = SocketAddr::new(self.bind, self.port);
            let socket = UdpSocket::bind(server_addr).unwrap();
            let public_addr = SocketAddr::new(self.ip, self.port);
            let server_config = ServerConfig::new(
                MAX_CLIENTS,
                PROTOCOL_ID,
                public_addr,
                ServerAuthentication::Unsecure,
            );

            let connection_config = RenetConnectionConfig {
                send_channels_config,
                receive_channels_config,
                ..Default::default()
            };

            primary_window.single_mut().title = "Server".to_string();

            RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
        };
        world.insert_resource(server);
        world
            .resource_mut::<NextState<GameState>>()
            .set(GameState::PreGame);
        SpawnPlayer {
            color: PlayerColor::get(0),
            network_owner: NetworkOwner(SERVER_ID),
        }
        .write(world);
        let mut mm_state = world.resource_mut::<MatchmakingState>();
        mm_state.lobby = Some(EphemeralMatchmakingLobby {
            ip: self.ip.to_string(),
            name: self.server_name,
            player_capacity: MAX_CLIENTS as u8,
            slots_occupied: 1,
            auto_restart: true,
            has_password: false,
            last_updated: 0,
        });
        mm_state.lobby_public = self.ip.is_global_unstable();
    }
}

#[derive(Debug, Default, Clone)]
pub struct Disconnect;

impl Command for Disconnect {
    fn write(self, world: &mut World) {
        debug!("Disconnecting");
        if let Some(mut server) = world.get_resource_mut::<RenetServer>() {
            server.disconnect_clients();
            world
                .resource_mut::<NextState<GameState>>()
                .set(GameState::MainMenu);
        }

        if let Some(mut client) = world.get_resource_mut::<RenetClient>() {
            client.disconnect();
            world
                .resource_mut::<NextState<GameState>>()
                .set(GameState::MainMenu);
        }
        world.resource_mut::<Players>().reset();
        world.remove_resource::<RenetServer>();
        world.remove_resource::<RenetClient>();
    }
}

fn apply_message_size_to_channels(channels: &mut [ChannelConfig]) {
    channels.iter_mut().for_each(|c| {
        if let ChannelConfig::Unreliable(c) = c {
            c.max_message_size = MAX_MESSAGE_SIZE;
            c.packet_budget = MAX_MESSAGE_SIZE * 2;
        }
    });
}

pub trait IpExt {
    /// This is the same implementation as the currently unstable [`IpAddr::is_global()`],
    /// re-implemented as an extension so we can use it in stable rust.
    fn is_global_unstable(&self) -> bool;
}

impl IpExt for Ipv4Addr {
    /// This is the same implementation as the currently unstable [`Ipv4Addr::is_global()`],
    /// re-implemented as an extension so we can use it in stable rust.
    fn is_global_unstable(&self) -> bool {
        !(self.octets()[0] == 0 // "This network"
            || self.is_private()
            || self.octets()[0] == 100 && (self.octets()[1] & 0b1100_0000 == 0b0100_0000)
            || self.is_loopback()
            || self.is_link_local()
            // addresses reserved for future protocols (`192.0.0.0/24`)
            || (self.octets()[0] == 192 && self.octets()[1] == 0 && self.octets()[2] == 0)
            || self.is_documentation()
            || self.octets()[0] == 198 && (self.octets()[1] & 0xfe) == 18
            || self.octets()[0] & 240 == 240 && !self.is_broadcast()
            || self.is_broadcast())
    }
}

impl IpExt for Ipv6Addr {
    /// This is the same implementation as the currently unstable [`Ipv6Addr::is_global()`],
    /// re-implemented as an extension so we can use it in stable rust.
    fn is_global_unstable(&self) -> bool {
        !(self.is_unspecified()
            || self.is_loopback()
            // IPv4-mapped Address (`::ffff:0:0/96`)
            || matches!(self.segments(), [0, 0, 0, 0, 0, 0xffff, _, _])
            // IPv4-IPv6 Translat. (`64:ff9b:1::/48`)
            || matches!(self.segments(), [0x64, 0xff9b, 1, _, _, _, _, _])
            // Discard-Only Address Block (`100::/64`)
            || matches!(self.segments(), [0x100, 0, 0, 0, _, _, _, _])
            // IETF Protocol Assignments (`2001::/23`)
            || (matches!(self.segments(), [0x2001, b, _, _, _, _, _, _] if b < 0x200)
            && !(
            // Port Control Protocol Anycast (`2001:1::1`)
            u128::from_be_bytes(self.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0001
                // Traversal Using Relays around NAT Anycast (`2001:1::2`)
                || u128::from_be_bytes(self.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0002
                // AMT (`2001:3::/32`)
                || matches!(self.segments(), [0x2001, 3, _, _, _, _, _, _])
                // AS112-v6 (`2001:4:112::/48`)
                || matches!(self.segments(), [0x2001, 4, 0x112, _, _, _, _, _])
                // ORCHIDv2 (`2001:20::/28`)
                || matches!(self.segments(), [0x2001, b, _, _, _, _, _, _] if (0x20..=0x2F).contains(&b))
        ))
            || ((self.segments()[0] == 0x2001) && (self.segments()[1] == 0xdb8))
            || (self.segments()[0] & 0xfe00) == 0xfc00
            || (self.segments()[0] & 0xffc0) == 0xfe80)
    }
}

impl IpExt for IpAddr {
    fn is_global_unstable(&self) -> bool {
        match self {
            IpAddr::V4(ip) => ip.is_global_unstable(),
            IpAddr::V6(ip) => ip.is_global_unstable(),
        }
    }
}
