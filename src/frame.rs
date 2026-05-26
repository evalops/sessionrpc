use bytes::Bytes;

use crate::{FrameSeq, LeaseEpoch, SessionId, StreamId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frame {
    session_id: SessionId,
    stream_id: StreamId,
    seq: FrameSeq,
    lease_epoch: LeaseEpoch,
    token_count: Option<u64>,
    kind: FrameKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrameKind {
    Data(Bytes),
    Cancel,
    Open,
    End,
    Ping,
}

impl Frame {
    pub fn data(
        session_id: SessionId,
        stream_id: StreamId,
        seq: FrameSeq,
        lease_epoch: LeaseEpoch,
        payload: Bytes,
    ) -> Self {
        Self {
            session_id,
            stream_id,
            seq,
            lease_epoch,
            token_count: None,
            kind: FrameKind::Data(payload),
        }
    }

    pub fn data_with_tokens(
        session_id: SessionId,
        stream_id: StreamId,
        seq: FrameSeq,
        lease_epoch: LeaseEpoch,
        payload: Bytes,
        token_count: u64,
    ) -> Self {
        Self {
            session_id,
            stream_id,
            seq,
            lease_epoch,
            token_count: Some(token_count),
            kind: FrameKind::Data(payload),
        }
    }

    pub fn control(
        session_id: SessionId,
        stream_id: StreamId,
        seq: FrameSeq,
        lease_epoch: LeaseEpoch,
        kind: FrameKind,
    ) -> Self {
        Self {
            session_id,
            stream_id,
            seq,
            lease_epoch,
            token_count: None,
            kind,
        }
    }

    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    pub fn stream_id(&self) -> StreamId {
        self.stream_id
    }

    pub fn seq(&self) -> FrameSeq {
        self.seq
    }

    pub fn lease_epoch(&self) -> LeaseEpoch {
        self.lease_epoch
    }

    pub fn kind(&self) -> &FrameKind {
        &self.kind
    }

    pub fn token_count(&self) -> Option<u64> {
        self.token_count
    }

    pub fn payload(&self) -> Option<Bytes> {
        match &self.kind {
            FrameKind::Data(payload) => Some(payload.clone()),
            _ => None,
        }
    }

    pub fn payload_len(&self) -> usize {
        match &self.kind {
            FrameKind::Data(payload) => payload.len(),
            _ => 0,
        }
    }
}
