# Rømer Chain

Rømer Chain reimagines blockchain infrastructure by putting market makers at the center of network operations. Through our unique Proof of Physics consensus mechanism and native FIX protocol support, we create natural regional advantages that protect local market makers while ensuring true physical decentralization.

## Project Overview

This repository contains the implementation of Rømer Chain, consisting of two main components:

- **Validator Node**: Built on Commonware primitives, our validator implementation combines blockchain consensus with market making operations.
- **Sequencer**: A high-performance FIX protocol gateway that enables seamless integration with existing trading infrastructure.

## Architecture

Rømer Chain is built as a Rust workspace with three main crates:

- `validator`: The core validator node implementation
- `sequencer`: FIX protocol integration and order sequencing
- `common`: Shared utilities and types

## Development Prerequisites

- Rust toolchain (stable channel)
- Protocol Buffers compiler

### Setting Up Development Environment

1. Install Rust using rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install additional dependencies:

On Ubuntu/Debian:
```bash
sudo apt update
sudo apt install build-essential cmake protobuf-compiler
```

On macOS:
```bash
brew install cmake protobuf
```

3. Clone the repository:
```bash
git clone https://github.com/romer-network/romer.git
cd romer
```

4. Build all components:
```bash
cargo build
```

## Crate Structure

### validator
The validator crate implements our Proof of Physics consensus mechanism and market making primitives. It handles:
- Network participation and block production
- Geographic validation
- Market making operations
- Hardware attestation

### sequencer
The sequencer crate provides FIX protocol support including:
- FIX message parsing and validation
- Order sequencing
- Market data distribution
- Session management

### common
Shared functionality between validator and sequencer:
- Cryptographic primitives
- Network types
- Configuration management
- Common utilities

## Contributing

We welcome contributions that align with our goal of building professional market making infrastructure. Please review our [contribution guidelines](CONTRIBUTING.md) before submitting pull requests.

### Development Workflow

1. Create a feature branch from `main`
2. Implement changes with appropriate tests
3. Ensure all tests pass and code is formatted
4. Submit a pull request with detailed description

### Code Style

We follow standard Rust formatting conventions. Before submitting code:

```bash
cargo fmt
cargo clippy
```

## License

Apache 2.0

## Contact

For development questions: [GitHub Issues](https://github.com/romer-network/romer/issues)
For market maker inquiries: @Hariseldon23 on Telegram



