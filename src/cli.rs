use crate::game_manager::GameState;
use crate::network::commands::NetworkCommandsExt;
use crate::network::DEFAULT_PORT;
use bevy::prelude::*;
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr};

pub struct CliPlugin;

impl Plugin for CliPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Cli::parse());
        app.add_systems((cli_system,).in_schedule(OnEnter(GameState::MainMenu)));
    }
}

#[derive(Debug, Parser, PartialEq, Resource)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    port: u16,
    /// Usually you can leave this to be the default.
    ///
    /// If you have multiple network adapters on your PC, you can use this to choose which to use
    /// by specifying the ip address assigned to that adapter on your local network.
    /// Use `ipconfig` (Windows) or `ifconfig` (Linux) to find the ip addresses assigned to your
    /// network adapters.
    ///
    /// By default it binds to 0.0.0.0 which tries to pick the adapter automatically.
    #[arg(short, long, default_value_t = Ipv4Addr::new(0,0,0,0).into())]
    bind: IpAddr,
    /// Start a server and listen for connections at this IP address.
    /// This should be your public IP address if you want your server to be public.
    #[arg(short, long)]
    listen: Option<IpAddr>,
    /// Connect to the server at this IP address
    #[arg(short, long)]
    connect: Option<IpAddr>,
}

fn cli_system(mut commands: Commands, settings: Res<Cli>) {
    if let Some(host_on_ip) = settings.listen {
        commands.listen(host_on_ip, settings.bind, settings.port);
    } else if let Some(join_ip) = settings.connect {
        commands.connect(join_ip, settings.bind, settings.port);
    }
}
