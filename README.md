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
cargo run -- --ip 27.33.41.4 --port 8000
```

The production environment utilizes Commonware's tokio runtime, which provides:

File System Storage: Data is persistently stored on disk using a write-ahead log journal with automatic compaction. The storage system implements atomic commits and provides crash recovery capabilities. Storage operations are optimized for production workloads with configurable sync intervals.

Real Clock Time: The node uses actual system time for block production and consensus. This ensures proper coordination with other network participants across geographic regions. Network timeouts and consensus intervals operate on real wall-clock time.

Network Stack: The tokio runtime implements full TCP networking with connection pooling, backpressure handling, and automatic reconnection. Network messages are handled asynchronously with configurable buffers to prevent memory exhaustion.

