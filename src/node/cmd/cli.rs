// cmd.rs
use clap::{value_parser, Arg, Command};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

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

pub fn setup_clap_command() -> (String, SocketAddr, clap::ArgMatches) {
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
        .get_matches();

    let me = matches
        .get_one::<(String, SocketAddr)>("me")
        .expect("Invalid 'me' argument format");
    let (node_id, socket_addr) = (me.0.clone(), me.1);

    (node_id, socket_addr, matches)
}
