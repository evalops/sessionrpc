# Architecture

`sessionrpc` is organized around a protocol core that can run over multiple
transports while preserving inference-session state.

## Core Concepts

### SessionRegistry

`SessionRegistry` owns local session state. It opens sessions, resumes them after
client reconnects, refreshes GPU leases when a scheduler migrates work, and
validates inbound frame sequence numbers.

### GpuLease

`GpuLease` is scheduler-neutral placement metadata. It records the worker, GPU
device ordinal, model id, and lease epoch. A refreshed lease changes the epoch,
which lets old clients and stale front doors fail fast instead of sending work to
the wrong warm model state.

### Frame

`Frame` is the protocol envelope. Data frames carry `Bytes`; control frames model
stream lifecycle events like cancel, open, end, and ping. Each frame includes a
session id, stream id, sequence number, and lease epoch.

### FlowController

`FlowController` tracks byte credit per stream. It is intentionally independent
from any one transport so network adapters can connect it to HTTP/2 windows,
QUIC stream credit, WebRTC data-channel buffering, or fleet-specific admission
signals.

### Transport

The initial transport is an in-memory pair used for tests and examples. Network
transports should preserve the same frame semantics: ordered frames per stream,
bidirectional sending, explicit close/error behavior, and bounded buffering.

## Data Flow

1. A front door asks a scheduler for placement and receives a `GpuLease`.
2. The front door opens or resumes a `SessionRegistry` entry.
3. Clients and workers exchange `Frame` values through a transport.
4. The registry validates the frame lease epoch and per-stream sequence cursor.
5. The flow controller reserves byte credit before payload frames enter the
   session.
6. If the scheduler migrates or renews placement, the registry refreshes the
   lease and rejects frames carrying stale epochs.

## Extension Points

- QUIC transport adapter for low-latency service-to-service streaming.
- WebRTC data-channel adapter for browser or edge clients.
- Codec layer for binary headers plus zero-copy payload bodies.
- Durable session store for multi-front-door reconnects.
- Scheduler trait that can be implemented by Kubernetes, Slurm, Ray, or a custom
  GPU allocator.
- Auth hooks for binding session ids and resume tokens to callers.
