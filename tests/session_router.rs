use bytes::Bytes;
use sessionrpc::{
    ClientId, Frame, FrameSeq, GpuLease, PlacementRequest, SessionRouter, SessionRpcError,
    StaticGpuScheduler, StreamId,
};

#[test]
fn router_opens_sessions_from_scheduler_placement_and_routes_frames() {
    let scheduler = StaticGpuScheduler::new(vec![GpuLease::new("worker-a", 0, "llama-70b", 11)]);
    let mut router = SessionRouter::new(scheduler, 1024);
    let opened = router
        .open(
            ClientId::new("client-a"),
            PlacementRequest::new("llama-70b").with_required_bytes(70_000_000_000),
        )
        .unwrap();

    let routed = router
        .route_inbound(Frame::data(
            opened.session_id,
            StreamId::new(1),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"prompt"),
        ))
        .unwrap();

    assert_eq!(opened.lease.worker_id, "worker-a");
    assert_eq!(routed.lease, opened.lease);
    assert_eq!(routed.next_inbound_seq, FrameSeq::new(1));
}

#[test]
fn router_rejects_payloads_without_advancing_the_session_cursor() {
    let scheduler = StaticGpuScheduler::new(vec![GpuLease::new("worker-a", 0, "llama-70b", 11)]);
    let mut router = SessionRouter::new(scheduler, 4);
    let opened = router
        .open(
            ClientId::new("client-a"),
            PlacementRequest::new("llama-70b"),
        )
        .unwrap();

    let err = router
        .route_inbound(Frame::data(
            opened.session_id,
            StreamId::new(1),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"too-large"),
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

    router.replenish(StreamId::new(1), 8);
    let routed = router
        .route_inbound(Frame::data(
            opened.session_id,
            StreamId::new(1),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"fits-now"),
        ))
        .unwrap();

    assert_eq!(routed.next_inbound_seq, FrameSeq::new(1));
}

#[test]
fn router_refreshes_lease_targets_without_losing_resume_state() {
    let scheduler = StaticGpuScheduler::new(vec![
        GpuLease::new("worker-a", 0, "llama-70b", 11),
        GpuLease::new("worker-b", 1, "llama-70b", 12),
    ]);
    let mut router = SessionRouter::new(scheduler, 1024);
    let opened = router
        .open(
            ClientId::new("client-a"),
            PlacementRequest::new("llama-70b"),
        )
        .unwrap();

    router
        .route_inbound(Frame::data(
            opened.session_id,
            StreamId::new(1),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"first"),
        ))
        .unwrap();

    let refreshed = router.refresh_lease(opened.session_id).unwrap();
    let resumed = router
        .resume(opened.session_id, ClientId::new("client-reconnected"))
        .unwrap();

    assert_eq!(refreshed.lease.worker_id, "worker-b");
    assert_eq!(resumed.next_inbound_seq(StreamId::new(1)), FrameSeq::new(1));
    assert_eq!(resumed.lease, refreshed.lease);
}
