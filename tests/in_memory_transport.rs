use bytes::Bytes;
use sessionrpc::{
    ClientId, Frame, FrameSeq, GpuLease, SessionRegistry, StreamId, in_memory_transport_pair,
};

#[tokio::test]
async fn in_memory_transport_streams_frames_bidirectionally() {
    let mut registry = SessionRegistry::default();
    let opened = registry.open_session(
        ClientId::new("client-a"),
        GpuLease::new("worker-a", 2, "vision-model", 100),
    );
    let (mut client, mut worker) = in_memory_transport_pair(8);

    client
        .send(Frame::data(
            opened.session_id,
            StreamId::new(1),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"image-tile"),
        ))
        .await
        .unwrap();

    worker
        .send(Frame::data(
            opened.session_id,
            StreamId::new(1),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"classification"),
        ))
        .await
        .unwrap();

    assert_eq!(
        worker.recv().await.unwrap().payload(),
        Some(Bytes::from_static(b"image-tile"))
    );
    assert_eq!(
        client.recv().await.unwrap().payload(),
        Some(Bytes::from_static(b"classification"))
    );
}
