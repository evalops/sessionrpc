# QUIC Transport

`QuicFrameTransport` is backed by Quinn. It implements `FrameTransport`, passes
the shared conformance harness, and uses `FrameCodec` as its wire boundary.

Each `send` opens a QUIC unidirectional stream, writes one encoded frame, and
finishes the stream. Each `recv` accepts one peer-opened unidirectional stream
and decodes the frame from it.

## 0-RTT Resume

`QuicClient::connect_0rtt` attempts Quinn's early-data path. If the local Quinn
endpoint has resumption state for the server, the returned
`QuicResumeConnection` can send frames before waiting for the resumed handshake
to complete.

```rust
let mut resumed = client.connect_0rtt().await?;
if resumed.attempted_0rtt() {
    resumed.transport_mut().send(frame).await?;
    let accepted = resumed.zero_rtt_accepted().await?;
}
```

0-RTT can be replayed by the network. Application code should only send
idempotent frames on this path, such as resume probes, cursor sync, and
deduplicated frame retransmits.

## Test Fixture

`QuicTestServer` creates a local self-signed Quinn server for integration tests.
It is intentionally part of the public API for transport implementors and binding
authors who need a local conformance target.
