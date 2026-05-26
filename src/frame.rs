use bytes::Bytes;

use crate::{FrameSeq, LeaseEpoch, SessionId, StreamId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frame {
    session_id: SessionId,
    stream_id: StreamId,
    seq: FrameSeq,
    lease_epoch: LeaseEpoch,
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
