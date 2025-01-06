use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "Rømer Chain",
    author = "Justin Trollip",
    version = "0.1.0",
    about = "The Layer 1 Built for Market Makers"
)]
pub struct NodeCliArgs {

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
}