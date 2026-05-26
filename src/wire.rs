use bytes::Bytes;

use crate::{Frame, FrameKind, FrameSeq, LeaseEpoch, SessionId, SessionRpcError, StreamId};

const MAGIC: &[u8; 4] = b"SRP1";
const VERSION: u8 = 1;
const HEADER_LEN: usize = 50;

#[derive(Clone, Debug)]
pub struct FrameCodec {
    max_payload_len: usize,
}

impl Default for FrameCodec {
    fn default() -> Self {
        Self {
            max_payload_len: 16 * 1024 * 1024,
        }
    }
}

impl FrameCodec {
    pub fn with_max_payload_len(max_payload_len: usize) -> Self {
        Self { max_payload_len }
    }

    pub fn encode(&self, frame: &Frame) -> Result<Bytes, SessionRpcError> {
        let payload = frame.payload().unwrap_or_default();
        if payload.len() > self.max_payload_len || payload.len() > u32::MAX as usize {
            return Err(SessionRpcError::FrameTooLarge(payload.len()));
        }

        let mut encoded = Vec::with_capacity(HEADER_LEN + payload.len());
        encoded.extend_from_slice(MAGIC);
        encoded.push(VERSION);
        encoded.push(kind_to_u8(frame.kind()));
        encoded.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        encoded.extend_from_slice(&frame.session_id().to_bytes());
        encoded.extend_from_slice(&frame.stream_id().get().to_be_bytes());
        encoded.extend_from_slice(&frame.seq().get().to_be_bytes());
        encoded.extend_from_slice(&frame.lease_epoch().get().to_be_bytes());
        encoded.extend_from_slice(&payload);

        Ok(Bytes::from(encoded))
    }

    pub fn decode(&self, encoded: &[u8]) -> Result<Frame, SessionRpcError> {
        if encoded.len() < HEADER_LEN {
            return Err(SessionRpcError::TruncatedFrame {
                needed: HEADER_LEN,
                actual: encoded.len(),
            });
        }

        if &encoded[..4] != MAGIC {
            return Err(SessionRpcError::InvalidFrameMagic);
        }

        let actual_version = encoded[4];
        if actual_version != VERSION {
            return Err(SessionRpcError::UnsupportedFrameVersion {
                supported: VERSION,
                actual: actual_version,
            });
        }

        let kind = encoded[5];
        let payload_len =
            u32::from_be_bytes(encoded[6..10].try_into().expect("slice len")) as usize;
        if payload_len > self.max_payload_len {
            return Err(SessionRpcError::FrameTooLarge(payload_len));
        }

        let needed = HEADER_LEN + payload_len;
        if encoded.len() < needed {
            return Err(SessionRpcError::TruncatedFrame {
                needed,
                actual: encoded.len(),
            });
        }

        let session_id = SessionId::from_bytes(encoded[10..26].try_into().expect("slice len"));
        let stream_id = StreamId::new(u64::from_be_bytes(
            encoded[26..34].try_into().expect("slice len"),
        ));
        let seq = FrameSeq::new(u64::from_be_bytes(
            encoded[34..42].try_into().expect("slice len"),
        ));
        let lease_epoch = LeaseEpoch::new(u64::from_be_bytes(
            encoded[42..50].try_into().expect("slice len"),
        ));
        let payload = Bytes::copy_from_slice(&encoded[50..needed]);

        let frame = match kind {
            0 => Frame::data(session_id, stream_id, seq, lease_epoch, payload),
            1 => Frame::control(session_id, stream_id, seq, lease_epoch, FrameKind::Cancel),
            2 => Frame::control(session_id, stream_id, seq, lease_epoch, FrameKind::Open),
            3 => Frame::control(session_id, stream_id, seq, lease_epoch, FrameKind::End),
            4 => Frame::control(session_id, stream_id, seq, lease_epoch, FrameKind::Ping),
            unknown => return Err(SessionRpcError::UnknownFrameKind(unknown)),
        };

        Ok(frame)
    }
}

fn kind_to_u8(kind: &FrameKind) -> u8 {
    match kind {
        FrameKind::Data(_) => 0,
        FrameKind::Cancel => 1,
        FrameKind::Open => 2,
        FrameKind::End => 3,
        FrameKind::Ping => 4,
    }
}
