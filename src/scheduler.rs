use crate::{GpuLease, SessionId, SessionRpcError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementRequest {
    pub model_id: String,
    pub required_bytes: Option<u64>,
}

impl PlacementRequest {
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            required_bytes: None,
        }
    }

    pub fn with_required_bytes(mut self, required_bytes: u64) -> Self {
        self.required_bytes = Some(required_bytes);
        self
    }
}

pub trait GpuScheduler {
    fn allocate(&mut self, request: &PlacementRequest) -> Result<GpuLease, SessionRpcError>;

    fn refresh(
        &mut self,
        _session_id: SessionId,
        current: &GpuLease,
    ) -> Result<GpuLease, SessionRpcError> {
        self.allocate(&PlacementRequest::new(current.model_id.clone()))
    }
}

#[derive(Clone, Debug)]
pub struct StaticGpuScheduler {
    leases: Vec<GpuLease>,
    next: usize,
}

impl StaticGpuScheduler {
    pub fn new(leases: Vec<GpuLease>) -> Self {
        Self { leases, next: 0 }
    }
}

impl GpuScheduler for StaticGpuScheduler {
    fn allocate(&mut self, request: &PlacementRequest) -> Result<GpuLease, SessionRpcError> {
        for offset in 0..self.leases.len() {
            let index = (self.next + offset) % self.leases.len();
            let lease = &self.leases[index];
            if lease.model_id == request.model_id {
                self.next = (index + 1) % self.leases.len();
                return Ok(lease.clone());
            }
        }

        Err(SessionRpcError::PlacementUnavailable {
            model_id: request.model_id.clone(),
        })
    }
}
