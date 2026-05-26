use std::slice;

use sessionrpc::{
    SESSIONRPC_FRAME_DATA, SessionRpcBuffer, SessionRpcBytes, SessionRpcDecodedFrame,
    SessionRpcFrameView, SessionRpcStatus, sessionrpc_buffer_free, sessionrpc_decode_frame,
    sessionrpc_decoded_frame_free, sessionrpc_encode_frame,
};

#[test]
fn c_binding_encodes_and_decodes_wire_frames() {
    let session_id = [7_u8; 16];
    let payload = b"hello from c";
    let trace_context = b"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00";
    let frame = SessionRpcFrameView {
        session_id,
        stream_id: 9,
        seq: 3,
        lease_epoch: 11,
        kind: SESSIONRPC_FRAME_DATA,
        payload: SessionRpcBytes {
            ptr: payload.as_ptr(),
            len: payload.len(),
        },
        has_token_count: true,
        token_count: 5,
        trace_context: SessionRpcBytes {
            ptr: trace_context.as_ptr(),
            len: trace_context.len(),
        },
    };
    let mut encoded = SessionRpcBuffer::default();

    let status = unsafe { sessionrpc_encode_frame(&frame, &mut encoded) };

    assert_eq!(status, SessionRpcStatus::Ok);
    assert!(!encoded.ptr.is_null());
    assert!(encoded.len > payload.len());

    let mut decoded = SessionRpcDecodedFrame::default();
    let status = unsafe { sessionrpc_decode_frame(encoded.ptr, encoded.len, &mut decoded) };

    assert_eq!(status, SessionRpcStatus::Ok);
    assert_eq!(decoded.session_id, session_id);
    assert_eq!(decoded.stream_id, 9);
    assert_eq!(decoded.seq, 3);
    assert_eq!(decoded.lease_epoch, 11);
    assert_eq!(decoded.kind, SESSIONRPC_FRAME_DATA);
    assert!(decoded.has_token_count);
    assert_eq!(decoded.token_count, 5);
    assert_eq!(
        unsafe { slice::from_raw_parts(decoded.payload.ptr, decoded.payload.len) },
        payload
    );
    assert_eq!(
        unsafe { slice::from_raw_parts(decoded.trace_context.ptr, decoded.trace_context.len) },
        trace_context
    );

    unsafe {
        sessionrpc_decoded_frame_free(&mut decoded);
        sessionrpc_buffer_free(&mut encoded);
    }
    assert!(decoded.payload.ptr.is_null());
    assert!(decoded.trace_context.ptr.is_null());
    assert!(encoded.ptr.is_null());
}
