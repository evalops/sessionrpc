use bytes::Bytes;
use sessionrpc::{
    Frame, FrameSeq, LeaseEpoch, QuicClient, QuicTestServer, StreamId,
    assert_transport_conformance_pair,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn quic_transport_satisfies_the_transport_contract() {
    let server = QuicTestServer::bind_local().await.unwrap();
    let client = QuicClient::new(server.addr(), server.certificate_der()).unwrap();
    let (server_transport, client_transport) =
        tokio::try_join!(server.accept(), client.connect()).unwrap();

    assert_transport_conformance_pair(client_transport, server_transport)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn quic_client_can_send_frames_on_a_zero_rtt_resume_path() {
    let server = QuicTestServer::bind_local().await.unwrap();
    let client = QuicClient::new(server.addr(), server.certificate_der()).unwrap();
    let (mut server_transport, mut client_transport) =
        tokio::try_join!(server.accept(), client.connect()).unwrap();

    let warm_frame = Frame::data(
        sessionrpc::SessionId::new(),
        StreamId::new(99),
        FrameSeq::new(0),
        LeaseEpoch::new(1),
        Bytes::from_static(b"warm resumption ticket"),
    );
    server_transport.send(warm_frame.clone()).await.unwrap();
    assert_eq!(client_transport.recv().await.unwrap(), warm_frame);
    client_transport.close();
    server_transport.close();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let frame = Frame::data(
        sessionrpc::SessionId::new(),
        StreamId::new(1),
        FrameSeq::new(0),
        LeaseEpoch::new(1),
        Bytes::from_static(b"resume without waiting"),
    );
    let server_future = async {
        let mut server_transport = server.accept().await.unwrap();
        server_transport.recv().await.unwrap()
    };
    let client_future = async {
        let mut resumed = client.connect_0rtt().await.unwrap();
        assert!(resumed.attempted_0rtt());
        resumed.transport_mut().send(frame.clone()).await.unwrap();
        resumed.zero_rtt_accepted().await.unwrap()
    };

    let (received, accepted) = tokio::join!(server_future, client_future);
    assert_eq!(received, frame);
    assert!(accepted);
}
