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

    #[error(
        "insufficient credit on stream {stream_id:?}: requested {requested}, available {available}"
    )]
    InsufficientCredit {
        stream_id: StreamId,
        requested: usize,
        available: usize,
    },

    #[error("invalid frame magic")]
    InvalidFrameMagic,

    #[error("unsupported frame version: supported {supported}, got {actual}")]
    UnsupportedFrameVersion { supported: u8, actual: u8 },

    #[error("unknown frame kind {0}")]
    UnknownFrameKind(u8),

    #[error("truncated frame: needed {needed} bytes, got {actual}")]
    TruncatedFrame { needed: usize, actual: usize },

    #[error("frame payload is too large: {0} bytes")]
    FrameTooLarge(usize),

    #[error("no GPU placement is available for model {model_id}")]
    PlacementUnavailable { model_id: String },

    #[error("transport is closed")]
    TransportClosed,
}
