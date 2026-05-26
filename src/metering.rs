use crate::{FrameSeq, LeaseEpoch, SessionId, StreamId};

pub trait MeteringSink {
    fn record_frame(&mut self, event: MeteringEvent);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeteringEvent {
    pub session_id: SessionId,
    pub stream_id: StreamId,
    pub seq: FrameSeq,
    pub lease_epoch: LeaseEpoch,
    pub worker_id: String,
    pub device_ordinal: u32,
    pub model_id: String,
    pub payload_bytes: u64,
    pub tokens: u64,
    pub session_seconds: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MeteringSnapshot {
    pub frames: u64,
    pub payload_bytes: u64,
    pub tokens: u64,
    pub session_seconds: u64,
}

#[derive(Clone, Debug, Default)]
pub struct NoopMeter;

impl MeteringSink for NoopMeter {
    fn record_frame(&mut self, _event: MeteringEvent) {}
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryMeter {
    events: Vec<MeteringEvent>,
}

impl InMemoryMeter {
    pub fn events(&self) -> &[MeteringEvent] {
        &self.events
    }

    pub fn snapshot(&self) -> MeteringSnapshot {
        self.events
            .iter()
            .fold(MeteringSnapshot::default(), |mut snapshot, event| {
                snapshot.frames += 1;
                snapshot.payload_bytes += event.payload_bytes;
                snapshot.tokens += event.tokens;
                snapshot.session_seconds += event.session_seconds;
                snapshot
            })
    }
}

impl MeteringSink for InMemoryMeter {
    fn record_frame(&mut self, event: MeteringEvent) {
        self.events.push(event);
    }
}
