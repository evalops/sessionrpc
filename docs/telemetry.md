# Telemetry

`sessionrpc` carries W3C `traceparent` context on frames and records span-shaped
events after routing succeeds. The crate keeps the core exporter-neutral: users
can bridge `FrameSpan` into OpenTelemetry through their preferred tracing stack.

```rust
use bytes::Bytes;
use sessionrpc::{
    ClientId, Frame, FrameSeq, GpuLease, InMemoryFrameTracer, PlacementRequest,
    SessionRouter, StaticGpuScheduler, StreamId, TraceContext,
};

let scheduler = StaticGpuScheduler::new(vec![
    GpuLease::new("worker-a", 0, "llama-70b", 1),
]);
let mut router = SessionRouter::with_tracer(
    scheduler,
    1024,
    InMemoryFrameTracer::default(),
);
let opened = router
    .open(ClientId::new("client-a"), PlacementRequest::new("llama-70b"))
    .unwrap();

let trace = TraceContext::new(
    "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
);

router
    .route_inbound(
        Frame::data(
            opened.session_id,
            StreamId::new(1),
            FrameSeq::new(0),
            opened.lease_epoch,
            Bytes::from_static(b"hello trace"),
        )
        .with_trace_context(trace),
    )
    .unwrap();

assert_eq!(router.tracer().spans()[0].name, "sessionrpc.frame.route");
```

## Span Fields

`FrameSpan` contains:

- `trace_context`: optional W3C `traceparent`;
- session id, stream id, sequence, and lease epoch;
- worker id, GPU ordinal, and model id;
- payload bytes and token count.

`FrameCodec` preserves trace context in the wire format so a span can follow a
frame across client, front door, scheduler, and worker boundaries.
