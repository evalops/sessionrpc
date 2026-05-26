use std::{ptr, slice, str};

use bytes::Bytes;

use crate::{
    Frame, FrameCodec, FrameKind, FrameSeq, LeaseEpoch, SessionId, StreamId, TraceContext,
};

pub const SESSIONRPC_FRAME_DATA: u8 = 0;
pub const SESSIONRPC_FRAME_CANCEL: u8 = 1;
pub const SESSIONRPC_FRAME_OPEN: u8 = 2;
pub const SESSIONRPC_FRAME_END: u8 = 3;
pub const SESSIONRPC_FRAME_PING: u8 = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionRpcStatus {
    Ok = 0,
    InvalidArgument = 1,
    DecodeError = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SessionRpcBytes {
    pub ptr: *const u8,
    pub len: usize,
}

impl Default for SessionRpcBytes {
    fn default() -> Self {
        Self {
            ptr: ptr::null(),
            len: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SessionRpcBuffer {
    pub ptr: *mut u8,
    pub len: usize,
    pub capacity: usize,
}

impl Default for SessionRpcBuffer {
    fn default() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SessionRpcFrameView {
    pub session_id: [u8; 16],
    pub stream_id: u64,
    pub seq: u64,
    pub lease_epoch: u64,
    pub kind: u8,
    pub payload: SessionRpcBytes,
    pub has_token_count: bool,
    pub token_count: u64,
    pub trace_context: SessionRpcBytes,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SessionRpcDecodedFrame {
    pub session_id: [u8; 16],
    pub stream_id: u64,
    pub seq: u64,
    pub lease_epoch: u64,
    pub kind: u8,
    pub payload: SessionRpcBytes,
    pub payload_capacity: usize,
    pub has_token_count: bool,
    pub token_count: u64,
    pub trace_context: SessionRpcBytes,
    pub trace_context_capacity: usize,
}

impl Default for SessionRpcDecodedFrame {
    fn default() -> Self {
        Self {
            session_id: [0; 16],
            stream_id: 0,
            seq: 0,
            lease_epoch: 0,
            kind: SESSIONRPC_FRAME_DATA,
            payload: SessionRpcBytes::default(),
            payload_capacity: 0,
            has_token_count: false,
            token_count: 0,
            trace_context: SessionRpcBytes::default(),
            trace_context_capacity: 0,
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sessionrpc_encode_frame(
    frame: *const SessionRpcFrameView,
    out: *mut SessionRpcBuffer,
) -> SessionRpcStatus {
    if frame.is_null() || out.is_null() {
        return SessionRpcStatus::InvalidArgument;
    }

    let view = unsafe { *frame };
    let payload = match unsafe { view.payload.as_slice() } {
        Ok(payload) => payload,
        Err(status) => return status,
    };
    let trace_context = match unsafe { view.trace_context.as_slice() } {
        Ok(trace_context) => trace_context,
        Err(status) => return status,
    };

    let frame = match frame_from_view(view, payload, trace_context) {
        Ok(frame) => frame,
        Err(status) => return status,
    };
    let encoded = match FrameCodec::default().encode(&frame) {
        Ok(encoded) => encoded,
        Err(_) => return SessionRpcStatus::InvalidArgument,
    };

    unsafe {
        *out = SessionRpcBuffer::from_vec(encoded.to_vec());
    }
    SessionRpcStatus::Ok
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sessionrpc_decode_frame(
    encoded: *const u8,
    encoded_len: usize,
    out: *mut SessionRpcDecodedFrame,
) -> SessionRpcStatus {
    if out.is_null() || (encoded.is_null() && encoded_len > 0) {
        return SessionRpcStatus::InvalidArgument;
    }

    let encoded = if encoded_len == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(encoded, encoded_len) }
    };
    let frame = match FrameCodec::default().decode(encoded) {
        Ok(frame) => frame,
        Err(_) => return SessionRpcStatus::DecodeError,
    };

    unsafe {
        *out = SessionRpcDecodedFrame::from_frame(frame);
    }
    SessionRpcStatus::Ok
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sessionrpc_buffer_free(buffer: *mut SessionRpcBuffer) {
    if buffer.is_null() {
        return;
    }

    let buffer = unsafe { &mut *buffer };
    unsafe {
        SessionRpcBuffer::free_parts(buffer.ptr, buffer.len, buffer.capacity);
    }
    *buffer = SessionRpcBuffer::default();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sessionrpc_decoded_frame_free(frame: *mut SessionRpcDecodedFrame) {
    if frame.is_null() {
        return;
    }

    let frame = unsafe { &mut *frame };
    unsafe {
        SessionRpcBuffer::free_parts(
            frame.payload.ptr.cast_mut(),
            frame.payload.len,
            frame.payload_capacity,
        );
        SessionRpcBuffer::free_parts(
            frame.trace_context.ptr.cast_mut(),
            frame.trace_context.len,
            frame.trace_context_capacity,
        );
    }
    *frame = SessionRpcDecodedFrame::default();
}

impl SessionRpcBytes {
    unsafe fn as_slice<'a>(self) -> Result<&'a [u8], SessionRpcStatus> {
        if self.ptr.is_null() && self.len > 0 {
            return Err(SessionRpcStatus::InvalidArgument);
        }
        if self.len == 0 {
            return Ok(&[]);
        }
        Ok(unsafe { slice::from_raw_parts(self.ptr, self.len) })
    }
}

impl SessionRpcBuffer {
    fn from_vec(mut bytes: Vec<u8>) -> Self {
        if bytes.is_empty() {
            return Self::default();
        }

        let buffer = Self {
            ptr: bytes.as_mut_ptr(),
            len: bytes.len(),
            capacity: bytes.capacity(),
        };
        std::mem::forget(bytes);
        buffer
    }

    unsafe fn free_parts(ptr: *mut u8, len: usize, capacity: usize) {
        if ptr.is_null() {
            return;
        }
        unsafe {
            drop(Vec::from_raw_parts(ptr, len, capacity));
        }
    }
}

impl SessionRpcDecodedFrame {
    fn from_frame(frame: Frame) -> Self {
        let payload = frame.payload().unwrap_or_default().to_vec();
        let trace_context = frame
            .trace_context()
            .map(|context| context.traceparent().as_bytes().to_vec())
            .unwrap_or_default();
        let payload = SessionRpcBuffer::from_vec(payload);
        let trace_context = SessionRpcBuffer::from_vec(trace_context);

        Self {
            session_id: frame.session_id().to_bytes(),
            stream_id: frame.stream_id().get(),
            seq: frame.seq().get(),
            lease_epoch: frame.lease_epoch().get(),
            kind: kind_to_u8(frame.kind()),
            payload: SessionRpcBytes {
                ptr: payload.ptr,
                len: payload.len,
            },
            payload_capacity: payload.capacity,
            has_token_count: frame.token_count().is_some(),
            token_count: frame.token_count().unwrap_or_default(),
            trace_context: SessionRpcBytes {
                ptr: trace_context.ptr,
                len: trace_context.len,
            },
            trace_context_capacity: trace_context.capacity,
        }
    }
}

fn frame_from_view(
    view: SessionRpcFrameView,
    payload: &[u8],
    trace_context: &[u8],
) -> Result<Frame, SessionRpcStatus> {
    let session_id = SessionId::from_bytes(view.session_id);
    let stream_id = StreamId::new(view.stream_id);
    let seq = FrameSeq::new(view.seq);
    let lease_epoch = LeaseEpoch::new(view.lease_epoch);
    let payload = Bytes::copy_from_slice(payload);
    let frame = match view.kind {
        SESSIONRPC_FRAME_DATA if view.has_token_count => Frame::data_with_tokens(
            session_id,
            stream_id,
            seq,
            lease_epoch,
            payload,
            view.token_count,
        ),
        SESSIONRPC_FRAME_DATA => Frame::data(session_id, stream_id, seq, lease_epoch, payload),
        SESSIONRPC_FRAME_CANCEL => {
            Frame::control(session_id, stream_id, seq, lease_epoch, FrameKind::Cancel)
        }
        SESSIONRPC_FRAME_OPEN => {
            Frame::control(session_id, stream_id, seq, lease_epoch, FrameKind::Open)
        }
        SESSIONRPC_FRAME_END => {
            Frame::control(session_id, stream_id, seq, lease_epoch, FrameKind::End)
        }
        SESSIONRPC_FRAME_PING => {
            Frame::control(session_id, stream_id, seq, lease_epoch, FrameKind::Ping)
        }
        _ => return Err(SessionRpcStatus::InvalidArgument),
    };

    if trace_context.is_empty() {
        return Ok(frame);
    }

    let trace_context = str::from_utf8(trace_context)
        .map_err(|_| SessionRpcStatus::InvalidArgument)?
        .to_string();
    Ok(frame.with_trace_context(TraceContext::new(trace_context)))
}

fn kind_to_u8(kind: &FrameKind) -> u8 {
    match kind {
        FrameKind::Data(_) => SESSIONRPC_FRAME_DATA,
        FrameKind::Cancel => SESSIONRPC_FRAME_CANCEL,
        FrameKind::Open => SESSIONRPC_FRAME_OPEN,
        FrameKind::End => SESSIONRPC_FRAME_END,
        FrameKind::Ping => SESSIONRPC_FRAME_PING,
    }
}
