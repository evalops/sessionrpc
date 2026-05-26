use crate::{
    AcceptedFrame, ClientId, FlowController, Frame, GpuScheduler, PlacementRequest, ResumedSession,
    SessionId, SessionRegistry, SessionRpcError, StreamId,
};

#[derive(Debug)]
pub struct SessionRouter<S> {
    scheduler: S,
    registry: SessionRegistry,
    flow: FlowController,
}

impl<S> SessionRouter<S>
where
    S: GpuScheduler,
{
    pub fn new(scheduler: S, default_stream_credit: usize) -> Self {
        Self {
            scheduler,
            registry: SessionRegistry::default(),
            flow: FlowController::new(default_stream_credit),
        }
    }

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
        self.registry.accept_frame(frame)
    }

    pub fn replenish(&mut self, stream_id: StreamId, bytes: usize) {
        self.flow.replenish(stream_id, bytes);
    }
}
