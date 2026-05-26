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

### FrameCodec

`FrameCodec` converts frames to and from a deterministic binary representation.
It gives transport adapters a shared byte boundary while keeping the protocol
core independent from QUIC, HTTP/2, WebRTC, or any other specific transport.

### FlowController

`FlowController` tracks byte credit per stream. It is intentionally independent
from any one transport so network adapters can connect it to HTTP/2 windows,
QUIC stream credit, WebRTC data-channel buffering, or fleet-specific admission
signals.

### GpuScheduler and SessionRouter

`GpuScheduler` is the scheduler integration point. Implementations turn a
`PlacementRequest` into a `GpuLease`; they can be backed by Kubernetes, Slurm,
Ray, a custom allocator, or a static test fixture.

`SessionRouter` ties the pieces together. It allocates leases through a
scheduler, opens and resumes sessions in the registry, refreshes leases when
placement changes, reserves byte credit, validates inbound frames, and returns
the lease target that a front door should dispatch to.

### Transport

The initial transport is an in-memory pair used for tests and examples. Network
transports should preserve the same frame semantics: ordered frames per stream,
bidirectional sending, explicit close/error behavior, and bounded buffering.

## Data Flow

1. A front door asks `SessionRouter` to open a session for a `PlacementRequest`.
2. The router asks its `GpuScheduler` for placement and receives a `GpuLease`.
3. The router opens or resumes a `SessionRegistry` entry.
4. Clients and workers exchange `Frame` values through a transport.
5. The flow controller reserves byte credit before payload frames enter the
   session.
6. The registry validates the frame lease epoch and per-stream sequence cursor.
7. If the scheduler migrates or renews placement, the router refreshes the lease
   and stale epochs are rejected before dispatch.

## Extension Points

- QUIC transport adapter for low-latency service-to-service streaming.
- WebRTC data-channel adapter for browser or edge clients.
- Streaming service traits for request handlers and worker-side dispatch.
- Durable session store for multi-front-door reconnects.
- Production scheduler adapters for Kubernetes, Slurm, Ray, or custom GPU
  allocators.
- Auth hooks for binding session ids and resume tokens to callers.
