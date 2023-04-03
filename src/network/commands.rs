use crate::network::{NetworkOwner, MAX_CLIENTS, PROTOCOL_ID};
use crate::player::commands::SpawnPlayer;
use crate::player::PlayerColor;
use bevy::ecs::system::{Command, SystemState};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_replicon::prelude::*;
use bevy_replicon::renet::{
    ClientAuthentication, RenetConnectionConfig, ServerAuthentication, ServerConfig,
};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::SystemTime;

pub trait NetworkCommandsExt {
    fn connect(&mut self, ip: IpAddr, bind: IpAddr, port: u16);
    fn listen(&mut self, ip: IpAddr, bind: IpAddr, port: u16);
}

impl<'w, 's> NetworkCommandsExt for Commands<'w, 's> {
    fn connect(&mut self, ip: IpAddr, bind: IpAddr, port: u16) {
        self.add(Connect { bind, ip, port });
    }

    fn listen(&mut self, ip: IpAddr, bind: IpAddr, port: u16) {
        self.add(Listen { bind, port, ip });
    }
}

pub struct Connect {
    pub bind: IpAddr,
    pub ip: IpAddr,
    pub port: u16,
}

impl Command for Connect {
    fn write(self, world: &mut World) {
        let client = {
            let mut state = SystemState::<(
                Res<NetworkChannels>,
                Query<&mut Window, With<PrimaryWindow>>,
            )>::new(world);
            let (network_channels, mut primary_window) = state.get_mut(world);
            let receive_channels_config = network_channels.server_channels();
            let send_channels_config = network_channels.client_channels();
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
        world.insert_resource(client);
    }
}

pub struct Listen {
    pub bind: IpAddr,
    pub ip: IpAddr,
    pub port: u16,
}

impl Command for Listen {
    fn write(self, world: &mut World) {
        let server = {
            let mut state = SystemState::<(
                Res<NetworkChannels>,
                Query<&mut Window, With<PrimaryWindow>>,
            )>::new(world);
            let (network_channels, mut primary_window) = state.get_mut(world);
            let send_channels_config = network_channels.server_channels();
            let receive_channels_config = network_channels.client_channels();
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
        SpawnPlayer {
            color: PlayerColor::get(0),
            network_owner: NetworkOwner(SERVER_ID),
        }
        .write(world);
    }
}
