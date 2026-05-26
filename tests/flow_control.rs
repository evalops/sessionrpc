use bytes::Bytes;
use sessionrpc::{
    ClientId, FlowController, Frame, FrameKind, FrameSeq, GpuLease, SessionRegistry,
    SessionRpcError, StreamId,
};

#[test]
fn byte_credit_is_tracked_per_stream_and_can_be_replenished() {
    let mut registry = SessionRegistry::default();
    let opened = registry.open_session(
        ClientId::new("client-a"),
        GpuLease::new("worker-a", 0, "token-model", 1),
    );
    let mut flow = FlowController::new(16);

    flow.reserve_frame(&Frame::data(
        opened.session_id,
        StreamId::new(1),
        FrameSeq::new(0),
        opened.lease_epoch,
        Bytes::from_static(b"1234567890"),
    ))
    .unwrap();

    let err = flow
        .reserve_frame(&Frame::data(
            opened.session_id,
            StreamId::new(1),
            FrameSeq::new(1),
            opened.lease_epoch,
            Bytes::from_static(b"1234567"),
        ))
        .unwrap_err();

    assert_eq!(
        err,
        SessionRpcError::InsufficientCredit {
            stream_id: StreamId::new(1),
            requested: 7,
            available: 6
        }
    );

    flow.replenish(StreamId::new(1), 4);
    flow.reserve_frame(&Frame::data(
        opened.session_id,
        StreamId::new(1),
        FrameSeq::new(1),
        opened.lease_epoch,
        Bytes::from_static(b"1234567"),
    ))
    .unwrap();

    assert_eq!(flow.available(StreamId::new(1)), 3);
    assert_eq!(flow.available(StreamId::new(2)), 16);
}

#[test]
fn control_frames_do_not_consume_byte_credit() {
    let mut registry = SessionRegistry::default();
    let opened = registry.open_session(
        ClientId::new("client-a"),
        GpuLease::new("worker-a", 0, "token-model", 1),
    );
    let mut flow = FlowController::new(0);

    flow.reserve_frame(&Frame::control(
        opened.session_id,
        StreamId::new(1),
        FrameSeq::new(0),
        opened.lease_epoch,
        FrameKind::Cancel,
    ))
    .unwrap();
}
