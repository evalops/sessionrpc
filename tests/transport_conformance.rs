use sessionrpc::{assert_transport_conformance, in_memory_transport_pair};

#[tokio::test]
async fn in_memory_transport_satisfies_the_transport_contract() {
    assert_transport_conformance(|| in_memory_transport_pair(8))
        .await
        .unwrap();
}
