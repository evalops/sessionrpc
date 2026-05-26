# Metering

`sessionrpc` meters at the protocol layer. A `SessionRouter` records a
`MeteringEvent` only after a frame has passed flow-control and session validation,
so rejected or stale frames are not counted as successful usage.

```rust
use bytes::Bytes;
use sessionrpc::{
    ClientId, Frame, FrameSeq, GpuLease, InMemoryMeter, PlacementRequest,
    SessionRouter, StaticGpuScheduler, StreamId,
};

let scheduler = StaticGpuScheduler::new(vec![
    GpuLease::new("worker-a", 0, "llama-70b", 1),
]);
let mut router = SessionRouter::with_meter(
    scheduler,
    1024,
    InMemoryMeter::default(),
);
let opened = router
    .open(ClientId::new("client-a"), PlacementRequest::new("llama-70b"))
    .unwrap();

router
    .route_inbound(Frame::data_with_tokens(
        opened.session_id,
        StreamId::new(1),
        FrameSeq::new(0),
        opened.lease_epoch,
        Bytes::from_static(b"hello world"),
        2,
    ))
    .unwrap();

assert_eq!(router.meter().snapshot().tokens, 2);
```

## What Is Counted

`MeteringEvent` records:

- session id, stream id, sequence, and lease epoch;
- worker id, GPU ordinal, and model id;
- payload bytes;
- token count carried by the frame;
- session seconds, currently initialized to `0` for frame events and reserved
  for lifecycle metering.

`FrameCodec` preserves token counts in the wire format so network transports can
carry metering metadata without side channels.
