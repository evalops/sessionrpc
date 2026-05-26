use crate::{
    AcceptedFrame, ClientId, FlowController, Frame, GpuScheduler, MeteringEvent, MeteringSink,
    NoopFrameTracer, NoopMeter, PlacementRequest, ResumedSession, SessionId, SessionRegistry,
    SessionRpcError, StreamId,
};
use crate::{FrameSpan, FrameTracer};

#[derive(Debug)]
pub struct SessionRouter<S, M = NoopMeter, T = NoopFrameTracer> {
    scheduler: S,
    registry: SessionRegistry,
    flow: FlowController,
    meter: M,
    tracer: T,
}

impl<S> SessionRouter<S, NoopMeter, NoopFrameTracer>
where
    S: GpuScheduler,
{
    pub fn new(scheduler: S, default_stream_credit: usize) -> Self {
        Self::with_meter_and_tracer(scheduler, default_stream_credit, NoopMeter, NoopFrameTracer)
    }

    pub fn with_meter<M>(
        scheduler: S,
        default_stream_credit: usize,
        meter: M,
    ) -> SessionRouter<S, M, NoopFrameTracer>
    where
        M: MeteringSink,
    {
        SessionRouter::with_meter_and_tracer(
            scheduler,
            default_stream_credit,
            meter,
            NoopFrameTracer,
        )
    }

    pub fn with_tracer<T>(
        scheduler: S,
        default_stream_credit: usize,
        tracer: T,
    ) -> SessionRouter<S, NoopMeter, T>
    where
        T: FrameTracer,
    {
        SessionRouter::with_meter_and_tracer(scheduler, default_stream_credit, NoopMeter, tracer)
    }

    pub fn with_meter_and_tracer<M, T>(
        scheduler: S,
        default_stream_credit: usize,
        meter: M,
        tracer: T,
    ) -> SessionRouter<S, M, T>
    where
        M: MeteringSink,
        T: FrameTracer,
    {
        SessionRouter {
            scheduler,
            registry: SessionRegistry::default(),
            flow: FlowController::new(default_stream_credit),
            meter,
            tracer,
        }
    }
}

impl<S, M, T> SessionRouter<S, M, T>
where
    S: GpuScheduler,
    M: MeteringSink,
    T: FrameTracer,
{
    pub fn open(
        &mut self,
        client_id: ClientId,
        placement: PlacementRequest,
    ) -> Result<crate::OpenedSession, SessionRpcError> {
        let lease = self.scheduler.allocate(&placement)?;
        Ok(self.registry.open_session(client_id, lease))
    }

    pub fn resume(
        &mut self,
        session_id: SessionId,
        client_id: ClientId,
    ) -> Result<ResumedSession, SessionRpcError> {
        self.registry.resume_session(session_id, client_id)
    }

    pub fn refresh_lease(
        &mut self,
        session_id: SessionId,
    ) -> Result<ResumedSession, SessionRpcError> {
        let current = self.registry.lease(session_id)?;
        let lease = self.scheduler.refresh(session_id, &current)?;
        self.registry.refresh_lease(session_id, lease)
    }

    pub fn route_inbound(&mut self, frame: Frame) -> Result<AcceptedFrame, SessionRpcError> {
        self.flow.reserve_frame(&frame)?;
        let payload_bytes = frame.payload_len() as u64;
        let tokens = frame.token_count().unwrap_or_default();
        let accepted = self.registry.accept_frame(frame.clone())?;

        self.meter.record_frame(MeteringEvent {
            session_id: frame.session_id(),
            stream_id: frame.stream_id(),
            seq: frame.seq(),
            lease_epoch: frame.lease_epoch(),
            worker_id: accepted.lease.worker_id.clone(),
            device_ordinal: accepted.lease.device_ordinal,
            model_id: accepted.lease.model_id.clone(),
            payload_bytes,
            tokens,
            session_seconds: 0,
        });
        self.tracer.record_span(FrameSpan {
            name: "sessionrpc.frame.route",
            trace_context: frame.trace_context().cloned(),
            session_id: frame.session_id(),
            stream_id: frame.stream_id(),
            seq: frame.seq(),
            lease_epoch: frame.lease_epoch(),
            worker_id: accepted.lease.worker_id.clone(),
            device_ordinal: accepted.lease.device_ordinal,
            model_id: accepted.lease.model_id.clone(),
            payload_bytes,
            tokens,
        });

        Ok(accepted)
    }

    pub fn replenish(&mut self, stream_id: StreamId, bytes: usize) {
        self.flow.replenish(stream_id, bytes);
    }

    pub fn meter(&self) -> &M {
        &self.meter
    }

    pub fn meter_mut(&mut self) -> &mut M {
        &mut self.meter
    }

    pub fn tracer(&self) -> &T {
        &self.tracer
    }

    pub fn tracer_mut(&mut self) -> &mut T {
        &mut self.tracer
    }
}
