use bytes::Bytes;
use sessionrpc::{
    ClientId, Frame, FrameKind, FrameSeq, GpuLease, SessionRegistry, SessionRpcError, StreamId,
};

#[test]
fn reconnect_preserves_gpu_affinity_and_resume_cursor() {
    let mut registry = SessionRegistry::default();
    let lease = GpuLease::new("worker-a", 3, "llama-70b", 7);
    let opened = registry.open_session(ClientId::new("browser-tab-1"), lease.clone());

    registry
        .accept_frame(Frame::data(
            opened.session_id,
            StreamId::new(11),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"first token"),
        ))
        .unwrap();

    let resumed = registry
        .resume_session(opened.session_id, ClientId::new("browser-tab-2"))
        .unwrap();

    assert_eq!(resumed.lease, lease);
    assert_eq!(
        resumed.next_inbound_seq(StreamId::new(11)),
        FrameSeq::new(1)
    );
    assert_eq!(resumed.client_id, ClientId::new("browser-tab-2"));
}

#[test]
fn routing_rejects_frames_with_stale_gpu_lease_epochs() {
    let mut registry = SessionRegistry::default();
    let opened = registry.open_session(
        ClientId::new("client-a"),
        GpuLease::new("worker-a", 0, "embedding-model", 41),
    );
    let refreshed = registry
        .refresh_lease(
            opened.session_id,
            GpuLease::new("worker-b", 1, "embedding-model", 42),
        )
        .unwrap();

    let err = registry
        .accept_frame(Frame::data(
            opened.session_id,
            StreamId::new(1),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"uses old worker"),
        ))
        .unwrap_err();

    assert_eq!(
        err,
        SessionRpcError::StaleLeaseEpoch {
            expected: refreshed.lease_epoch,
            actual: opened.lease_epoch
        }
    );
}

#[test]
fn streams_enforce_ordering_per_session_and_stream() {
    let mut registry = SessionRegistry::default();
    let opened = registry.open_session(
        ClientId::new("client-a"),
        GpuLease::new("worker-a", 0, "speech-model", 9),
    );

    registry
        .accept_frame(Frame::data(
            opened.session_id,
            StreamId::new(5),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"chunk-0"),
        ))
        .unwrap();

    let duplicate = registry
        .accept_frame(Frame::data(
            opened.session_id,
            StreamId::new(5),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"chunk-0-again"),
        ))
        .unwrap_err();

    assert_eq!(
        duplicate,
        SessionRpcError::OutOfOrderFrame {
            stream_id: StreamId::new(5),
            expected: FrameSeq::new(1),
            actual: FrameSeq::new(0)
        }
    );

    let gap = registry
        .accept_frame(Frame::control(
            opened.session_id,
            StreamId::new(6),
            FrameSeq::new(4),
            opened.lease_epoch,
            FrameKind::Cancel,
        ))
        .unwrap_err();

    assert_eq!(
        gap,
        SessionRpcError::OutOfOrderFrame {
            stream_id: StreamId::new(6),
            expected: FrameSeq::new(0),
            actual: FrameSeq::new(4)
        }
    );
}
