//! GPU-session-aware, bidirectional streaming RPC for stateful inference fleets.
//!
//! This crate is being built around a small protocol core, pluggable transports,
//! and scheduler-neutral session metadata.

mod error;
mod ffi;
mod flow;
mod frame;
mod gpu;
mod ids;
mod k8s;
mod metering;
mod quic;
mod router;
mod scheduler;
mod session;
mod telemetry;
mod transport;
mod wire;

pub use error::SessionRpcError;
pub use ffi::{
    SESSIONRPC_FRAME_CANCEL, SESSIONRPC_FRAME_DATA, SESSIONRPC_FRAME_END, SESSIONRPC_FRAME_OPEN,
    SESSIONRPC_FRAME_PING, SessionRpcBuffer, SessionRpcBytes, SessionRpcDecodedFrame,
    SessionRpcFrameView, SessionRpcStatus, sessionrpc_buffer_free, sessionrpc_decode_frame,
    sessionrpc_decoded_frame_free, sessionrpc_encode_frame,
};
pub use flow::FlowController;
pub use frame::{Frame, FrameKind};
pub use gpu::GpuLease;
pub use ids::{ClientId, FrameSeq, LeaseEpoch, SessionId, StreamId};
pub use k8s::{K8sRoutePolicy, StickyRouteDecision, StickyRouteTable};
pub use metering::{InMemoryMeter, MeteringEvent, MeteringSink, MeteringSnapshot, NoopMeter};
pub use quic::{QuicClient, QuicFrameTransport, QuicResumeConnection, QuicTestServer};
pub use router::SessionRouter;
pub use scheduler::{GpuScheduler, PlacementRequest, StaticGpuScheduler};
pub use session::{AcceptedFrame, OpenedSession, ResumedSession, SessionRegistry, SessionSnapshot};
pub use telemetry::{FrameSpan, FrameTracer, InMemoryFrameTracer, NoopFrameTracer, TraceContext};
pub use transport::{
    FrameTransport, InMemoryEndpoint, assert_transport_conformance,
    assert_transport_conformance_pair, in_memory_transport_pair,
};
pub use wire::FrameCodec;
