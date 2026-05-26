# sessionrpc

`sessionrpc` is a Rust framework for GPU-session-aware, bidirectional streaming
RPC between clients and stateful inference fleets.

It is meant to sit in the space between low-level transports like gRPC, QUIC, or
WebRTC and the scheduling layer that owns warm model state, GPU affinity, and
long-running inference sessions.

## Design goals

- Preserve session affinity across reconnects and transport upgrades.
- Stream typed control messages and binary frame payloads in both directions.
- Surface GPU placement and lease state without coupling users to one scheduler.
- Keep transports pluggable: in-memory first, network transports next.
- Make backpressure, cancellation, and resumability explicit in the protocol.

## Status

This repository is in active bootstrap. The first usable milestone is a Rust
library crate with an in-memory transport, session registry, frame protocol, and
examples that model stateful GPU inference flows.
