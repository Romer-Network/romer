use clap::{Parser, command, ValueEnum};
use std::net::SocketAddr;

// First, let's create an enum for the possible environments
#[derive(Debug, Clone, ValueEnum)]
pub enum Environment {
    /// Production environment using tokio runtime
    Production,
    /// Testing environment using deterministic runtime
    Testing,
    /// Local development environment with additional logging
    Development,
}

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
        help = "Specify the environment (production, testing, development)"
    )]
    pub environment: Environment,

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
    // Existing methods remain the same
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

    // Add a helper method to get appropriate runtime settings
    pub fn get_runtime_config(&self) -> RuntimeConfig {
        match self.environment {
            Environment::Production => RuntimeConfig::new_production(),
            Environment::Testing => RuntimeConfig::new_testing(),
            Environment::Development => RuntimeConfig::new_development(),
        }
    }
}

// You'll need to define this struct to hold runtime-specific configurations
pub struct RuntimeConfig {
    pub storage_partition: String,
    pub network_timeout: std::time::Duration,
    // Add other runtime-specific settings
}

impl RuntimeConfig {
    pub fn new_production() -> Self {
        Self {
            storage_partition: "prod".to_string(),
            network_timeout: std::time::Duration::from_secs(30),
            // Set production-appropriate values
        }
    }

    pub fn new_testing() -> Self {
        Self {
            storage_partition: "test".to_string(),
            network_timeout: std::time::Duration::from_secs(5),
            // Set testing-appropriate values
        }
    }

    pub fn new_development() -> Self {
        Self {
            storage_partition: "dev".to_string(),
            network_timeout: std::time::Duration::from_secs(10),
            // Set development-appropriate values
        }
    }
}