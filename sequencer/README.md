# Rømer Chain Sequencer

The Rømer Chain Sequencer serves as a bridge between traditional financial infrastructure and blockchain operations. It accepts standard FIX 4.2 messages from market makers, validates and processes these messages, and creates blocks of transactions that can be executed on the blockchain.

## System Architecture

The sequencer is built around three core domains that work together to process trading operations:

### Session Management

The session layer handles FIX connection lifecycles and ensures secure, reliable communication with market makers. At its heart is the SessionManager, which coordinates three key components:

The Session struct represents an individual FIX connection and maintains:
- Connection state (Connecting, Active, Terminated, etc.)
- Message sequence numbers
- Heartbeat timing
- Market maker identification

The SessionAuthenticator provides blockchain-style security through Commonware storage integration:
- Manages market maker registration in persistent storage
- Verifies BLS signatures during logon
- Handles key rotation and revocation
- Provides administrative interfaces for market maker management

The MarketMaker storage schema includes:
- Sender comp ID (primary identifier)
- BLS public key for authentication
- Registration timestamp
- Current status and permissions
- Historical key rotation records

The session code maintains strict state transitions:
```
Connecting -> Authenticating -> Active -> ResyncRequired -> Active -> Disconnecting -> Terminated
```

### FIX Processing

The FIX processing layer handles message parsing, validation, and initial processing. It consists of several integrated components:

The FixParser handles the raw mechanics of FIX message processing:
- Parses raw FIX messages using the fefix library
- Validates FIX syntax and required fields
- Extracts message content into structured data
- Enforces message type support restrictions

Currently supported FIX messages:
1. Administrative Messages:
   - Logon (A)
   - Heartbeat (0)
   - Test Request (1)
   - Resend Request (2)
   - Sequence Reset (4)
   - Logout (5)

2. Application Messages:
   - New Order Single (D)
   Additional message types will be added as the system evolves.

The message validation follows a multi-stage process:
1. Syntax validation (correct FIX format)
2. Field validation (required fields present)
3. Message type validation (ensuring message type is supported)
4. Value validation (field contents are valid)
5. Business validation (order parameters make sense)

### Block Creation

The block creation layer transforms validated FIX messages into blockchain blocks through a coordinated process involving several components:

The BatchManager collects validated messages until either:
- 500 messages have accumulated, or
- 500 milliseconds have elapsed
This dual trigger system ensures both efficient processing and predictable latency.

The BlockTimer provides precise timing control:
- Maintains the 500ms block window
- Compensates for system scheduling delays
- Ensures consistent block creation timing
- Monitors timing performance

The BlockBuilder creates the final blocks by:
- Assembling messages into a block structure
- Creating block headers with metadata
- Calculating message merkle roots
- Maintaining block sequence numbers

### Network Layer

The network layer provides essential connectivity for both testing and production:

Connection Management:
- Accepts incoming TCP connections for FIX sessions
- Manages connection lifecycles
- Handles network errors and reconnection
- Provides connection monitoring and statistics

Session Establishment:
- Processes FIX logon sequences
- Validates initial connection parameters
- Establishes heartbeat intervals
- Manages session authentication

Message Handling:
- Receives raw FIX messages
- Routes messages to appropriate sessions
- Handles message fragmentation and reassembly
- Provides flow control and backpressure

The network layer is crucial for:
- Testing with real FIX clients
- Validating session management
- Verifying timing and performance
- Ensuring proper error handling

### Data Flow

Let's follow a message through the system:

1. The network layer accepts a FIX connection and message

2. The SessionManager:
   - Validates the session is active
   - Checks sequence numbers
   - Updates session state

3. The FIX layer:
   - Verifies message type is supported
   - Parses and validates the message
   - Extracts message content

4. The BatchManager:
   - Adds the message to current batch
   - Monitors batch size and timing
   - Triggers block creation when needed

5. The BlockBuilder:
   - Creates a new block from the batch
   - Calculates necessary hashes
   - Prepares the block for execution

## Code Organization

The codebase is organized into four main directories:

```
src/
├── session/           # Session management
│   ├── manager.rs    # Session lifecycle handling
│   ├── state.rs      # Session state and transitions
│   └── auth.rs       # BLS authentication
│
├── fix/              # FIX processing
│   ├── parser.rs     # Message parsing
│   ├── validator.rs  # Message validation
│   └── types.rs      # FIX message structures
│
├── block/           # Block creation
│   ├── batch.rs     # Message batching
│   ├── timer.rs     # Block timing
│   └── builder.rs   # Block construction
│
└── network/         # Network handling
    ├── listener.rs  # Connection acceptance
    ├── session.rs   # FIX session handling
    └── message.rs   # Message routing
```

Each component is designed to be:
- Self-contained with clear responsibilities
- Connected through well-defined interfaces
- Independently testable
- Easily maintainable

## Component Communication

The components communicate through tokio channels, which provide:
- Asynchronous operation
- Backpressure handling
- Clean error propagation
- Thread safety

The channel flow follows this pattern:
```
Network Layer -> SessionManager -> FIX Parser -> BatchManager -> BlockBuilder
```

## Development Status

Current Implementation:
- Basic session management with in-memory storage
- Initial FIX message parsing and validation
- Block creation and timing infrastructure

Pending Implementation:
1. Commonware Storage Integration:
   - Market maker registration storage
   - Key management and rotation
   - Administrative interfaces

2. Message Type Support:
   - Complete supported message validation
   - Additional message type handlers
   - Version compatibility checks

3. Network Layer:
   - TCP connection handling
   - Session establishment
   - Message routing
   - Error handling

## Testing Strategy

The sequencer can be tested at multiple levels:

Unit Tests:
- Individual component functionality
- Message parsing and validation
- Block creation logic
- Session state management

Integration Tests:
- Component interaction
- Message flow through system
- Timing and batching behavior
- Storage integration

Network Tests:
- FIX client connectivity
- Session establishment
- Message transmission
- Error handling

The network layer implementation is crucial for comprehensive testing with real FIX clients and validating the system under realistic conditions.

## Getting Started

To understand the codebase, start with:

1. Session Management: Examine session/state.rs to understand the basic session lifecycle and state transitions.

2. FIX Processing: Look at fix/parser.rs to see how raw FIX messages are handled and validated.

3. Block Creation: Review block/batch.rs to understand how messages are collected and transformed into blocks.

4. Network Layer: Once implemented, network/listener.rs will show how FIX connections are established and managed.

Remember that every component in the system maintains its own state and runs its own background task, coordinated through channels and shared state structures.