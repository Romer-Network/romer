# RØMER Chain Node Configuration Guide

When running a RØMER Chain node, you must configure both your local environment and network settings to enable proper participation in the network's Proof of Physics consensus mechanism.

## Network Configuration

### Port Forwarding Setup
Before starting your node, you need to configure port forwarding on your router to enable location validation. This allows other validators to verify your physical presence in your claimed geographic region.

To set up port forwarding:

1. Access your router's administration interface by entering its IP address in your web browser (typically 192.168.0.1 or 192.168.1.1)

2. Locate the port forwarding section (sometimes called "Virtual Server" or "Port Mapping")

3. Create a new port forwarding rule with these settings:
   - External Port: 8000 (or your chosen node port)
   - Internal Port: 8000 (must match external port)
   - Protocol: TCP
   - Internal IP: Your node's local IP address
   - Description: "RØMER Chain Node"

4. Enable the rule and save your changes

5. Verify your port forwarding setup by using a port checking service

Port forwarding is crucial for the network's security model - without it, your node cannot participate in geographic validation, which would prevent you from participating in consensus.

### Obtain External IP address

Get your Public IP address from https://www.whatismyip.com/

## Environment Options

### Production Environment
```bash
cargo run -- -e production --ip 27.33.41.4 --port 8000
```

The production environment utilizes Commonware's tokio runtime, which provides:

File System Storage: Data is persistently stored on disk using a write-ahead log journal with automatic compaction. The storage system implements atomic commits and provides crash recovery capabilities. Storage operations are optimized for production workloads with configurable sync intervals.

Real Clock Time: The node uses actual system time for block production and consensus. This ensures proper coordination with other network participants across geographic regions. Network timeouts and consensus intervals operate on real wall-clock time.

Network Stack: The tokio runtime implements full TCP networking with connection pooling, backpressure handling, and automatic reconnection. Network messages are handled asynchronously with configurable buffers to prevent memory exhaustion.

### Development Environment 
```bash
cargo run -- -e development --ip 127.0.0.1 --port 8000
```

The development environment uses Commonware's deterministic runtime, providing:

Simulated Storage: Instead of writing to disk, the storage system maintains data in memory with simulated persistence. This allows for faster testing cycles and predictable storage behavior. The storage system still maintains ACID properties but without actual disk I/O.

Deterministic Time: Time advancement is controlled programmatically rather than using system time. This enables reproducible testing of time-dependent behaviors like consensus rounds and network timeouts. Tests can explicitly advance time to trigger specific behaviors.

Simulated Network: The network layer simulates message delivery with configurable latency and packet loss. This allows testing network partition scenarios and consensus behavior under adverse conditions. Network characteristics remain consistent across test runs.


## Additional Configuration

Each environment supports these additional flags:

```
--genesis               # Start as a genesis node
--bootstrappers         # Connect to existing network
--log-level             # Set logging verbosity
```

## Impact on Consensus

The choice of environment affects how Simplex Consensus operates:

Production uses real network latency and system time for view changes and leader election. Geographic distribution of nodes impacts actual consensus timing.

Testing provides reproducible consensus runs by controlling message delivery and time advancement. This enables verification of consensus properties under specific conditions.

Development maintains consensus properties while providing additional introspection into the consensus process through enhanced logging and metrics.

## Resource Requirements

Each environment has different resource needs:

Production requires dedicated hardware meeting minimum specifications for CPU, memory, storage, and network bandwidth. Storage requires high-performance SSDs for journal persistence.

Testing can run with reduced resources since storage is simulated and network operations are lightweight. Memory requirements are higher since data is kept in RAM.

Development has similar requirements to production but needs additional storage space for debug logs and metrics data.

The environment choice fundamentally shapes how your node interacts with the network through Commonware's runtime abstractions. Choose based on your specific needs for persistence, determinism, and debugging capabilities.

Would you like me to expand on any aspect of how Commonware's runtimes behave in these environments? For example, I could elaborate on the storage implementation differences or consensus timing behaviors.