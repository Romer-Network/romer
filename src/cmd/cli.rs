use clap::{Parser, command};
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[command(
    name = "Rømer Chain",
    author = "Rømer Chain Development Team",
    version = "0.1.0",
    about = "A blockchain with physical infrastructure requirements"
)]
pub struct NodeCliArgs {
    /// Network address for this node in the format IP:PORT
    #[arg(
        short, 
        long,
        default_value = "127.0.0.1:8000",
        help = "The network address this node will listen on"
    )]
    pub address: SocketAddr,

    /// Designates this node as a genesis node
    #[arg(
        short,
        long,
        help = "Start this node as a genesis node"
    )]
    pub genesis: bool,

    /// Address of an existing node to bootstrap from
    #[arg(
        short,
        long,
        help = "Address of an existing node to connect to",
        required_unless_present = "genesis",
        conflicts_with = "genesis"
    )]
    pub bootstrap: Option<String>,

    /// Log level for node operation
    #[arg(
        short,
        long,
        default_value = "info",
        help = "Set the logging level",
        value_parser = ["error", "warn", "info", "debug", "trace"]
    )]
    pub log_level: String,
}

impl NodeCliArgs {
    pub fn get_log_level(&self) -> tracing::Level {
        match self.log_level.as_str() {
            "error" => tracing::Level::ERROR,
            "warn" => tracing::Level::WARN,
            "info" => tracing::Level::INFO,
            "debug" => tracing::Level::DEBUG,
            "trace" => tracing::Level::TRACE,
            _ => tracing::Level::INFO,
        }
    }

    pub fn get_bootstrap_addr(&self) -> Option<SocketAddr> {
        self.bootstrap
            .as_ref()
            .map(|addr| addr.parse().expect("Invalid bootstrap address"))
    }
}