use std::collections::HashMap;

use crate::{Frame, SessionRpcError, StreamId};

#[derive(Clone, Debug)]
pub struct FlowController {
    default_credit: usize,
    available: HashMap<StreamId, usize>,
}

impl FlowController {
    pub fn new(default_credit: usize) -> Self {
        Self {
            default_credit,
            available: HashMap::new(),
        }
    }

    pub fn reserve_frame(&mut self, frame: &Frame) -> Result<(), SessionRpcError> {
        let requested = frame.payload_len();
        if requested == 0 {
            return Ok(());
        }

        self.reserve(frame.stream_id(), requested)
    }

    pub fn reserve(
        &mut self,
        stream_id: StreamId,
        requested: usize,
    ) -> Result<(), SessionRpcError> {
        let available = self
            .available
            .entry(stream_id)
            .or_insert(self.default_credit);
        if requested > *available {
            return Err(SessionRpcError::InsufficientCredit {
                stream_id,
                requested,
                available: *available,
            });
        }

        *available -= requested;
        Ok(())
    }

    pub fn replenish(&mut self, stream_id: StreamId, bytes: usize) {
        let available = self
            .available
            .entry(stream_id)
            .or_insert(self.default_credit);
        *available += bytes;
    }

    pub fn available(&self, stream_id: StreamId) -> usize {
        self.available
            .get(&stream_id)
            .copied()
            .unwrap_or(self.default_credit)
    }
}
