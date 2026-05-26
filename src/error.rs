use crate::{FrameSeq, LeaseEpoch, SessionId, StreamId};

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SessionRpcError {
    #[error("unknown session {0}")]
    UnknownSession(SessionId),

    #[error("stale lease epoch: expected {expected:?}, got {actual:?}")]
    StaleLeaseEpoch {
        expected: LeaseEpoch,
        actual: LeaseEpoch,
    },

    #[error("out-of-order frame on stream {stream_id:?}: expected {expected:?}, got {actual:?}")]
    OutOfOrderFrame {
        stream_id: StreamId,
        expected: FrameSeq,
        actual: FrameSeq,
    },

    #[error("transport is closed")]
    TransportClosed,
}
