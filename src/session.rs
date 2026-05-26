use std::collections::HashMap;

use crate::{ClientId, Frame, FrameSeq, GpuLease, SessionId, SessionRpcError, StreamId};

#[derive(Debug, Default)]
pub struct SessionRegistry {
    sessions: HashMap<SessionId, SessionState>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenedSession {
    pub session_id: SessionId,
    pub client_id: ClientId,
    pub lease: GpuLease,
    pub lease_epoch: crate::LeaseEpoch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResumedSession {
    pub session_id: SessionId,
    pub client_id: ClientId,
    pub lease: GpuLease,
    pub lease_epoch: crate::LeaseEpoch,
    inbound_cursors: HashMap<StreamId, FrameSeq>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedFrame {
    pub session_id: SessionId,
    pub stream_id: StreamId,
    pub next_inbound_seq: FrameSeq,
    pub lease: GpuLease,
}

#[derive(Clone, Debug)]
struct SessionState {
    client_id: ClientId,
    lease: GpuLease,
    inbound_cursors: HashMap<StreamId, FrameSeq>,
}

impl SessionRegistry {
    pub fn open_session(&mut self, client_id: ClientId, lease: GpuLease) -> OpenedSession {
        let session_id = SessionId::new();
        let opened = OpenedSession {
            session_id,
            client_id: client_id.clone(),
            lease: lease.clone(),
            lease_epoch: lease.epoch,
        };

        self.sessions.insert(
            session_id,
            SessionState {
                client_id,
                lease,
                inbound_cursors: HashMap::new(),
            },
        );

        opened
    }

    pub fn resume_session(
        &mut self,
        session_id: SessionId,
        client_id: ClientId,
    ) -> Result<ResumedSession, SessionRpcError> {
        let state = self
            .sessions
            .get_mut(&session_id)
            .ok_or(SessionRpcError::UnknownSession(session_id))?;
        state.client_id = client_id.clone();

        Ok(state.snapshot(session_id, client_id))
    }

    pub fn refresh_lease(
        &mut self,
        session_id: SessionId,
        lease: GpuLease,
    ) -> Result<ResumedSession, SessionRpcError> {
        let state = self
            .sessions
            .get_mut(&session_id)
            .ok_or(SessionRpcError::UnknownSession(session_id))?;
        state.lease = lease;

        Ok(state.snapshot(session_id, state.client_id.clone()))
    }

    pub fn lease(&self, session_id: SessionId) -> Result<GpuLease, SessionRpcError> {
        let state = self
            .sessions
            .get(&session_id)
            .ok_or(SessionRpcError::UnknownSession(session_id))?;

        Ok(state.lease.clone())
    }

    pub fn accept_frame(&mut self, frame: Frame) -> Result<AcceptedFrame, SessionRpcError> {
        let state = self
            .sessions
            .get_mut(&frame.session_id())
            .ok_or(SessionRpcError::UnknownSession(frame.session_id()))?;

        if frame.lease_epoch() != state.lease.epoch {
            return Err(SessionRpcError::StaleLeaseEpoch {
                expected: state.lease.epoch,
                actual: frame.lease_epoch(),
            });
        }

        let expected = state
            .inbound_cursors
            .entry(frame.stream_id())
            .or_insert(FrameSeq::new(0));
        if frame.seq() != *expected {
            return Err(SessionRpcError::OutOfOrderFrame {
                stream_id: frame.stream_id(),
                expected: *expected,
                actual: frame.seq(),
            });
        }

        *expected = expected.next();

        Ok(AcceptedFrame {
            session_id: frame.session_id(),
            stream_id: frame.stream_id(),
            next_inbound_seq: *expected,
            lease: state.lease.clone(),
        })
    }
}

impl ResumedSession {
    pub fn next_inbound_seq(&self, stream_id: StreamId) -> FrameSeq {
        self.inbound_cursors
            .get(&stream_id)
            .copied()
            .unwrap_or_else(|| FrameSeq::new(0))
    }
}

impl SessionState {
    fn snapshot(&self, session_id: SessionId, client_id: ClientId) -> ResumedSession {
        ResumedSession {
            session_id,
            client_id,
            lease: self.lease.clone(),
            lease_epoch: self.lease.epoch,
            inbound_cursors: self.inbound_cursors.clone(),
        }
    }
}
