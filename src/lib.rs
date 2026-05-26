//! GPU-session-aware, bidirectional streaming RPC for stateful inference fleets.
//!
//! This crate is being built around a small protocol core, pluggable transports,
//! and scheduler-neutral session metadata.

mod error;
mod frame;
mod gpu;
mod ids;
mod session;
mod transport;

pub use error::SessionRpcError;
pub use frame::{Frame, FrameKind};
pub use gpu::GpuLease;
pub use ids::{ClientId, FrameSeq, LeaseEpoch, SessionId, StreamId};
pub use session::{AcceptedFrame, OpenedSession, ResumedSession, SessionRegistry};
pub use transport::{InMemoryEndpoint, in_memory_transport_pair};
