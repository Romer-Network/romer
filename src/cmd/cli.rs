use crate::config::runtime::RuntimeConfig;
use crate::config::runtime::RuntimeEnvironment;
use clap::{command, Parser, ValueEnum};
use std::net::SocketAddr;

// Remove the Environment enum since we'll use RuntimeEnvironment

#[derive(Parser, Debug)]
#[command(
    name = "Rømer Chain",
    author = "Rømer Chain Development Team",
    version = "0.1.0",
    about = "A blockchain with physical infrastructure requirements"
)]
pub struct NodeCliArgs {
    /// The environment to run the node in
    #[arg(
        short,
        long,
        value_enum,
        default_value = "development",
        help = "Specify the environment (production or development)"
    )]
    pub environment: RuntimeEnvironment, // Changed to use RuntimeEnvironment

    #[arg(
        long,
        default_value = "127.0.0.1",
        help = "The IP address this node will listen on"
    )]
    pub ip: String,

    #[arg(
        long,
        default_value = "8000",
        help = "The port number this node will listen on"
    )]
    pub port: u16,

    #[arg(
        short,
        long,
        help = "Addresses of existing nodes to connect to (format: seed@ip:port)",
        value_delimiter = ','
    )]
    pub bootstrappers: Option<Vec<String>>,

    /// Designates this node as a genesis node
    #[arg(short, long, help = "Start this node as a genesis node")]
    pub genesis: bool,

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
}
