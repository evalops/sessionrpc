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

This repository is in active bootstrap. The current crate includes:

- `SessionRouter` for scheduler-backed open, resume, lease refresh, and routing.
- `SessionRegistry` for affinity, reconnect state, and per-stream sequence
  cursors.
- `GpuScheduler`, `PlacementRequest`, and `GpuLease` for scheduler-neutral GPU
  placement.
- `Frame`, `FrameCodec`, and `FrameKind` for typed protocol frames and a binary
  wire boundary.
- `FlowController` for explicit per-stream byte credit.
- `MeteringSink` hooks for counting frames, payload bytes, tokens, and session
  time at the protocol layer.
- `FrameTracer` hooks and W3C `traceparent` propagation for OpenTelemetry-style
  frame spans.
- `FrameTransport`, `InMemoryEndpoint`, and a reusable conformance harness for
  local tests and future network transports.
- `QuicFrameTransport` built on Quinn, including a 0-RTT resume path for flaky
  mobile reconnects.

## Quick start

```bash
cargo test
cargo run --example inference_session
```

```rust
use bytes::Bytes;
use sessionrpc::{
    ClientId, Frame, FrameSeq, GpuLease, PlacementRequest, SessionRouter,
    StaticGpuScheduler, StreamId,
};

let scheduler = StaticGpuScheduler::new(vec![
    GpuLease::new("worker-a", 0, "llama-70b", 1),
]);
let mut router = SessionRouter::new(scheduler, 1024 * 1024);
let opened = router.open(
    ClientId::new("client-a"),
    PlacementRequest::new("llama-70b"),
)?;

let frame = Frame::data(
    opened.session_id,
    StreamId::new(1),
    FrameSeq::new(0),
    opened.lease_epoch,
    Bytes::from_static(b"prompt bytes"),
);

let route = router.route_inbound(frame)?;
assert_eq!(route.lease.worker_id, "worker-a");
```

See [docs/architecture.md](docs/architecture.md) for the current architecture,
[docs/wire-format.md](docs/wire-format.md) for the binary frame format, and
[docs/transport-conformance.md](docs/transport-conformance.md) for the transport
contract. See [docs/metering.md](docs/metering.md) for protocol-layer metering.
See [docs/telemetry.md](docs/telemetry.md) for frame tracing.
See [docs/quic.md](docs/quic.md) for the QUIC transport.
