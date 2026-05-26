use crate::{FrameSeq, LeaseEpoch, SessionId, StreamId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceContext {
    traceparent: String,
}

impl TraceContext {
    pub fn new(traceparent: impl Into<String>) -> Self {
        Self {
            traceparent: traceparent.into(),
        }
    }

    pub fn traceparent(&self) -> &str {
        &self.traceparent
    }
}

pub trait FrameTracer {
    fn record_span(&mut self, span: FrameSpan);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameSpan {
    pub name: &'static str,
    pub trace_context: Option<TraceContext>,
    pub session_id: SessionId,
    pub stream_id: StreamId,
    pub seq: FrameSeq,
    pub lease_epoch: LeaseEpoch,
    pub worker_id: String,
    pub device_ordinal: u32,
    pub model_id: String,
    pub payload_bytes: u64,
    pub tokens: u64,
}

#[derive(Clone, Debug, Default)]
pub struct NoopFrameTracer;

impl FrameTracer for NoopFrameTracer {
    fn record_span(&mut self, _span: FrameSpan) {}
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryFrameTracer {
    spans: Vec<FrameSpan>,
}

impl InMemoryFrameTracer {
    pub fn spans(&self) -> &[FrameSpan] {
        &self.spans
    }
}

impl FrameTracer for InMemoryFrameTracer {
    fn record_span(&mut self, span: FrameSpan) {
        self.spans.push(span);
    }
}
