// cmd.rs
use clap::{value_parser, Arg, Command};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use crate::types::ValidatorLocation;

pub struct AppConfig {
    pub bootstrappers: Vec<String>,
    pub me: (String, SocketAddr),
    pub participants: Vec<u64>,
    pub storage_dir: String,
    pub location: ValidatorLocation,
}

fn parse_me(value: &str) -> Result<(String, SocketAddr), String> {
    let mut parts = value.split('@');
    let node_id = parts.next().ok_or("Invalid format for 'me' argument")?;

    let port_and_addr = parts.next().ok_or("Invalid format for 'me' argument")?;
    let mut port_and_addr_parts = port_and_addr.split(':');
    let ip_addr_str = port_and_addr_parts
        .next()
        .ok_or("Invalid format for 'me' argument")?;
    let port_str = port_and_addr_parts
        .next()
        .ok_or("Invalid format for 'me' argument")?;

    let ip_addr: IpAddr = ip_addr_str.parse().map_err(|_| "Invalid IP address")?;
    let port: u16 = port_str.parse().map_err(|_| "Invalid port number")?;

    let socket_addr = SocketAddr::new(ip_addr, port);

    Ok((node_id.to_string(), socket_addr))
}

// cmd.rs
pub fn setup_clap_command() -> AppConfig {
    let matches = Command::new("romer")
        .about("generate secret logs and agree on their hash")
        .arg(
            Arg::new("bootstrappers")
                .long("bootstrappers")
                .required(false)
                .value_delimiter(',')
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("me")
                .long("me")
                .required(true)
                .value_parser(parse_me),
        )
        .arg(
            Arg::new("participants")
                .long("participants")
                .required(true)
                .value_delimiter(',')
                .value_parser(value_parser!(u64))
                .help("All participants (arbiter and contributors)"),
        )
        .arg(Arg::new("storage-dir").long("storage-dir").required(true))
        .arg(
            Arg::new("latitude")
                .long("latitude")
                .required(true)
                .value_parser(value_parser!(f64))
                .help("Validator's latitude coordinate (-90 to 90)")
        )
        .arg(
            Arg::new("longitude")
                .long("longitude")
                .required(true)
                .value_parser(value_parser!(f64))
                .help("Validator's longitude coordinate (-180 to 180)")
        )
        .get_matches();

    let me = matches
        .get_one::<(String, SocketAddr)>("me")
        .expect("Invalid 'me' argument format");
    let bootstrappers = matches
        .get_many::<String>("bootstrappers")
        .map(|b| b.cloned().collect())
        .unwrap_or_default();
    let participants = matches
        .get_many::<u64>("participants")
        .map(|p| p.cloned().collect())
        .expect("Please provide at least one participant");
    let storage_dir = matches
        .get_one::<String>("storage-dir")
        .expect("Please provide storage directory")
        .clone();
    let latitude = *matches.get_one::<f64>("latitude")
        .expect("Latitude is required");
    let longitude = *matches.get_one::<f64>("longitude")
        .expect("Longitude is required");
    
        let location = ValidatorLocation::new(latitude, longitude)
        .expect("Invalid validator location coordinates");

    AppConfig {
        bootstrappers,
        me: (me.0.clone(), me.1),
        participants,
        storage_dir,
        location,
    }
}
