# Rømer Chain
    ██████╗  ██████╗ ███╗   ███╗███████╗██████╗ 
    ██╔══██╗██╔═══██╗████╗ ████║██╔════╝██╔══██╗
    ██████╔╝██║   ██║██╔████╔██║█████╗  ██████╔╝
    ██╔══██╗██║   ██║██║╚██╔╝██║██╔══╝  ██╔══██╗
    ██║  ██║╚██████╔╝██║ ╚═╝ ██║███████╗██║  ██║
    ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝

## Prerequisites

You must have Rust installed on your system. If you haven't installed it yet, run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Building

Clone and build the project:

```bash
cargo build --release
```

## Running a Node

### Starting the Genesis Node
To start the genesis node in the network:

```bash
cargo run -- -a 127.0.0.1:8000 -g
```

### Command Options

The node command accepts these arguments:

```bash
--address      
--genesis
```

## Monitoring
`brew install prometheus`
`brew install grafana`
`brew services start grafana`

Go to 
http://localhost:3000

to access Grafana

Login with admin/admin

`prometheus --config.file=./prometheus.yaml`

This will start Prometheus at

http://localhost:9090

Add Prometheus as a Data Source to Grafana