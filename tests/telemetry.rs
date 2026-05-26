use bytes::Bytes;
use sessionrpc::{
    ClientId, Frame, FrameCodec, FrameSeq, GpuLease, InMemoryFrameTracer, PlacementRequest,
    SessionRouter, StaticGpuScheduler, StreamId, TraceContext,
};

#[test]
fn router_records_frame_spans_with_trace_context_and_routing_fields() {
    let scheduler = StaticGpuScheduler::new(vec![GpuLease::new("worker-a", 0, "llama-70b", 11)]);
    let mut router = SessionRouter::with_tracer(scheduler, 1024, InMemoryFrameTracer::default());
    let opened = router
        .open(
            ClientId::new("client-a"),
            PlacementRequest::new("llama-70b"),
        )
        .unwrap();
    let trace = TraceContext::new("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01");

    router
        .route_inbound(
            Frame::data_with_tokens(
                opened.session_id,
                StreamId::new(1),
                FrameSeq::new(0),
                opened.lease_epoch,
                Bytes::from_static(b"hello trace"),
                2,
            )
            .with_trace_context(trace.clone()),
        )
        .unwrap();

    let span = &router.tracer().spans()[0];
    assert_eq!(span.name, "sessionrpc.frame.route");
    assert_eq!(span.trace_context, Some(trace));
    assert_eq!(span.session_id, opened.session_id);
    assert_eq!(span.stream_id, StreamId::new(1));
    assert_eq!(span.seq, FrameSeq::new(0));
    assert_eq!(span.lease_epoch, opened.lease_epoch);
    assert_eq!(span.worker_id, "worker-a");
    assert_eq!(span.model_id, "llama-70b");
    assert_eq!(span.payload_bytes, 11);
    assert_eq!(span.tokens, 2);
}

#[test]
fn trace_context_roundtrips_through_the_wire_codec() {
    let codec = FrameCodec::default();
    let trace = TraceContext::new("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01");
    let frame = Frame::data(
        sessionrpc::SessionId::new(),
        StreamId::new(1),
        FrameSeq::new(0),
        sessionrpc::LeaseEpoch::new(1),
        Bytes::from_static(b"payload"),
    )
    .with_trace_context(trace);

    assert_eq!(codec.decode(&codec.encode(&frame).unwrap()).unwrap(), frame);
}
