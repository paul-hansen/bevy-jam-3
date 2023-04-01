use bevy::prelude::*;
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr};

const PORT: u16 = 4761;
const PROTOCOL_ID: u64 = 0;

pub struct CliPlugin;

impl Plugin for CliPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Cli::parse());
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
        public_ip: IpAddr,
    },
    Client {
        #[arg(short, long, default_value_t = Ipv4Addr::LOCALHOST.into())]
        ip: IpAddr,

        #[arg(short, long, default_value_t = PORT)]
        port: u16,
    },
}
