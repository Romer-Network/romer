# Rømer Chain

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
The Genesis node is responsible for the following:
* Creates and Signs the genesis block
* List of initial validators
* Network Protocol Parameters (regions, block size limits, consensus etc)
* Network Identifier

To start the genesis node in the network:

```bash
cargo run -- -a 127.0.0.1:8000 -g
```

### Joining an Existing Network

To connect to an existing network, you'll need to know the key and address of at least one running node. Then start your node with the bootstrappers flag:

```bash
cargo run --release -- node --key 5678 --region amsterdam --port 30304 --bootstrappers 1234@127.0.0.1:30303
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

## Consensus
Current implementation uses Simplex with a round robin leader selection. Once the network is working smoothly we'll then adjust Leader selection.

RomerSupervisor handles validator set management and leader selection.