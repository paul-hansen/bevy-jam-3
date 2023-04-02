use crate::player::{spawn_player, PlayerColor};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_replicon::prelude::*;
use bevy_replicon::renet::{
    ClientAuthentication, RenetConnectionConfig, ServerAuthentication, ServerConfig,
};
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::SystemTime;

const PORT: u16 = 4761;
const PROTOCOL_ID: u64 = 0;

pub struct CliPlugin;

impl Plugin for CliPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Cli::parse());
        app.add_startup_system(cli_system);
    }
}

#[derive(Debug, Parser, PartialEq, Resource)]
enum Cli {
    Server {
        #[arg(short, long, default_value_t = PORT)]
        port: u16,
        #[arg(short, long, default_value_t = Ipv4Addr::LOCALHOST.into())]
        bind_ip: IpAddr,
        #[arg(short, long, default_value_t = Ipv4Addr::LOCALHOST.into())]
        wan_ip: IpAddr,
    },
    Client {
        #[arg(short, long, default_value_t = Ipv4Addr::LOCALHOST.into())]
        ip: IpAddr,

        #[arg(short, long, default_value_t = PORT)]
        port: u16,
    },
}

fn cli_system(
    mut commands: Commands,
    settings: Res<Cli>,
    network_channels: Res<NetworkChannels>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    match *settings {
        Cli::Server {
            port,
            bind_ip,
            wan_ip,
        } => {
            let send_channels_config = network_channels.server_channels();
            let receive_channels_config = network_channels.client_channels();
            const MAX_CLIENTS: usize = 1;
            let current_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            let server_addr = SocketAddr::new(bind_ip, port);
            let socket = UdpSocket::bind(server_addr).unwrap();
            let public_addr = SocketAddr::new(wan_ip, port);
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

            let server =
                RenetServer::new(current_time, server_config, connection_config, socket).unwrap();

            spawn_player(PlayerColor::get(0), &mut commands, SERVER_ID);

            commands.insert_resource(server);
            primary_window.single_mut().title = "Server".to_string();
        }
        Cli::Client { port, ip } => {
            let receive_channels_config = network_channels.server_channels();
            let send_channels_config = network_channels.client_channels();
            let current_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            let client_id = current_time.as_millis() as u64;
            let server_addr = SocketAddr::new(ip, port);
            let socket = UdpSocket::bind(("0.0.0.0", 0)).expect("0.0.0.0 should be bindable");
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

            let client =
                RenetClient::new(current_time, socket, connection_config, authentication).unwrap();
            commands.insert_resource(client);
            primary_window.single_mut().title = "Client".to_string();
        }
    }
}
