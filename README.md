# RØMER Chain

RØMER Chain is a novel blockchain platform that implements Proof of Physics consensus through physical validator requirements and geographic validation. The system validates nodes through hardware attestation and network latency measurements against known reference points.

## Core Features

- **Physical Hardware Validation**: Ensures validators run on real hardware, not cloud instances
- **Geographic Validation**: Validates node locations through speed-of-light network latency measurements
- **Proof of Physics**: Novel consensus mechanism combining physical and network validations
- **Regional Protection**: Creates natural geographic advantages for local market makers

## Prerequisites

- Rust toolchain (latest stable version)
- Physical hardware (not a virtual machine)
- Network connectivity to DE-CIX Frankfurt (for latency validation)
- Storage space for the blockchain data

## Building from Source

```bash
# Clone the repository
git clone https://github.com/romer-network/romer.git
cd romer

# Build with optimizations
cargo build --release
```

## Running a Validator Node

RØMER requires at least 3 validator nodes to reach consensus. Each node requires specific geographic coordinates and must pass hardware and latency validations.

### Configuration Parameters

- `--me`: Your node identifier and network address (format: `id@ip:port`)
- `--participants`: Comma-separated list of participant IDs
- `--storage-dir`: Directory for blockchain data storage
- `--latitude`: Node's geographic latitude
- `--longitude`: Node's geographic longitude
- `--bootstrappers`: Address of bootstrap node (required for non-bootstrap nodes)

### Example Network Setup

Below are example commands to run a 4-node test network. Each command should be run in a separate terminal.

#### Bootstrap Node (Node 0)

```bash
# Unix/macOS
cargo run --release -- \
  --me 0@127.0.0.1:3000 \
  --participants 0,1,2,3 \
  --storage-dir ./data/log/0 \
  --latitude=-28.0167 \
  --longitude=153.4000

# Windows
cargo run --release -- --me 0@127.0.0.1:3000 --participants 0,1,2,3 --storage-dir data\romer_log\0 --latitude=-28.0167 --longitude=153.4000
```

#### Additional Nodes (1-3)

For each additional node, adjust the node ID, port, and storage directory:

```bash
# Unix/macOS
cargo run --release -- \
  --bootstrappers 0@127.0.0.1:3000 \
  --me NODE_ID@127.0.0.1:PORT \
  --participants 0,1,2,3 \
  --storage-dir ./data/log/NODE_ID \
  --latitude=-28.0167 \
  --longitude=153.4000

# Windows
cargo run --release -- --bootstrappers 0@127.0.0.1:3000 --me NODE_ID@127.0.0.1:PORT --participants 0,1,2,3 --storage-dir data\romer_log\NODE_ID --latitude=-28.0167 --longitude=153.4000
```

Replace NODE_ID with 1, 2, or 3, and PORT with 3001, 3002, or 3003 respectively.

## Validation Process

When a node starts, it performs two key validations:

1. **Hardware Validation**: Verifies the node is running on physical hardware
2. **Latency Validation**: Measures network latency to DE-CIX Frankfurt (80.81.192.3) and validates it against speed-of-light constraints based on the node's claimed location

The node will only join the network if both validations pass.

## Development Status

RØMER Chain is currently in active development. Key features like zero-knowledge proofs for hardware attestation and more sophisticated geographic validation are under development.

## License

Apache 

## Contributing

We welcome contributions to RØMER Chain. Please read our contributing guidelines before submitting pull requests.