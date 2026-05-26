# Transport Conformance

Every transport adapter should pass `assert_transport_conformance`.

```rust
use sessionrpc::{assert_transport_conformance, in_memory_transport_pair};

#[tokio::test]
async fn transport_satisfies_sessionrpc_contract() {
    assert_transport_conformance(|| in_memory_transport_pair(8))
        .await
        .unwrap();
}
```

The harness currently checks that a transport:

- sends frames bidirectionally;
- preserves FIFO order within a direction;
- preserves frame identity without mutating session, stream, sequence, lease, or
  payload fields;
- reports errors through `SessionRpcError`.

Future transport adapters should add their own transport-specific tests around
handshake behavior, reconnects, datagram or stream settings, browser lifecycle,
and security, then call the shared conformance harness as the common baseline.
