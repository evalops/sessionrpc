use crate::LeaseEpoch;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GpuLease {
    pub worker_id: String,
    pub device_ordinal: u32,
    pub model_id: String,
    pub epoch: LeaseEpoch,
}

impl GpuLease {
    pub fn new(
        worker_id: impl Into<String>,
        device_ordinal: u32,
        model_id: impl Into<String>,
        epoch: u64,
    ) -> Self {
        Self {
            worker_id: worker_id.into(),
            device_ordinal,
            model_id: model_id.into(),
            epoch: LeaseEpoch::new(epoch),
        }
    }
}
