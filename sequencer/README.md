# Rømer Chain Sequencer

The Rømer Chain Sequencer provides FIX protocol support for the network, enabling market makers to connect their existing trading systems directly to validator nodes. It handles FIX session management, message parsing, and order flow integration with the blockchain.

## Getting Started

Build the sequencer:
```bash
cargo build -p romer-sequencer
```

Run with default configuration:
```bash
cargo run -p romer-sequencer
```

Run with a custom config file:
```bash
cargo run -p romer-sequencer -- --config path/to/config.toml
```

## Configuration

The sequencer accepts a TOML configuration file with these settings:

```toml
[fix]
port = 9898
sender_comp_id = "ROMER"
version = "FIX.4.4"

[network]
listen_addr = "127.0.0.1"
```

## Features

The sequencer currently supports:
- FIX 4.4 protocol
- Session management
- Basic message validation
- Heartbeat monitoring
- Order acceptance

## Development

Run the test suite:
```bash
cargo test -p romer-sequencer
```

Check code formatting:
```bash
cargo fmt --all
cargo clippy -p romer-sequencer
```

## Architecture

The sequencer connects external FIX clients to the Rømer Chain network:

```
FIX Client -> Sequencer -> Validator Node -> Network
```

Messages flow through these stages:
1. FIX message reception
2. Protocol validation
3. Conversion to blockchain transactions
4. Submission to validator

## Contributing

Refer to the main project CONTRIBUTING.md for development guidelines.

## Status

The sequencer is in active development. Current focus areas:
- Session management robustness
- Full FIX message support
- Performance optimization
- Market data integration