use bytes::Bytes;
use sessionrpc::{
    ClientId, Frame, FrameSeq, GpuLease, InMemoryMeter, MeteringSnapshot, PlacementRequest,
    SessionRouter, SessionRpcError, StaticGpuScheduler, StreamId,
};

#[test]
fn router_records_frame_metering_after_successful_route() {
    let scheduler = StaticGpuScheduler::new(vec![GpuLease::new("worker-a", 0, "llama-70b", 11)]);
    let mut router = SessionRouter::with_meter(scheduler, 1024, InMemoryMeter::default());
    let opened = router
        .open(
            ClientId::new("client-a"),
            PlacementRequest::new("llama-70b"),
        )
        .unwrap();

    router
        .route_inbound(Frame::data_with_tokens(
            opened.session_id,
            StreamId::new(1),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"one two three"),
            3,
        ))
        .unwrap();

    assert_eq!(
        router.meter().snapshot(),
        MeteringSnapshot {
            frames: 1,
            payload_bytes: 13,
            tokens: 3,
            session_seconds: 0
        }
    );
}

#[test]
fn router_does_not_meter_rejected_frames() {
    let scheduler = StaticGpuScheduler::new(vec![GpuLease::new("worker-a", 0, "llama-70b", 11)]);
    let mut router = SessionRouter::with_meter(scheduler, 4, InMemoryMeter::default());
    let opened = router
        .open(
            ClientId::new("client-a"),
            PlacementRequest::new("llama-70b"),
        )
        .unwrap();

    let err = router
        .route_inbound(Frame::data_with_tokens(
            opened.session_id,
            StreamId::new(1),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"too-large"),
            3,
        ))
        .unwrap_err();

    assert_eq!(
        err,
        SessionRpcError::InsufficientCredit {
            stream_id: StreamId::new(1),
            requested: 9,
            available: 4
        }
    );
    assert_eq!(router.meter().snapshot(), MeteringSnapshot::default());
}
