use sessionrpc::{ClientId, GpuLease, K8sRoutePolicy, SessionRegistry, StickyRouteTable};

#[test]
fn registry_snapshot_produces_sticky_k8s_backend_decisions() {
    let mut registry = SessionRegistry::default();
    let opened = registry.open_session(
        ClientId::new("browser-tab-1"),
        GpuLease::new("worker-a", 0, "llama-70b", 7),
    );
    registry.open_session(
        ClientId::new("browser-tab-2"),
        GpuLease::new("worker-b", 1, "llama-70b", 3),
    );

    let policy = K8sRoutePolicy::new("inference", "sessionrpc-worker", 8080);
    let table = StickyRouteTable::from_registry(&registry, policy.clone());

    let route = table.route(opened.session_id).unwrap();
    assert_eq!(route.session_id, opened.session_id);
    assert_eq!(route.sticky_header, "x-sessionrpc-session");
    assert_eq!(route.sticky_key, opened.session_id.to_string());
    assert_eq!(route.backend_namespace, "inference");
    assert_eq!(route.backend_service, "sessionrpc-worker-worker-a");
    assert_eq!(route.backend_port, 8080);
    assert_eq!(route.worker_id, "worker-a");
    assert_eq!(route.device_ordinal, 0);
    assert_eq!(route.model_id, "llama-70b");
    assert_eq!(route.lease_epoch, opened.lease_epoch);

    registry
        .refresh_lease(
            opened.session_id,
            GpuLease::new("worker-c", 2, "llama-70b", 8),
        )
        .unwrap();

    let table = StickyRouteTable::from_registry(&registry, policy);
    let refreshed = table.route(opened.session_id).unwrap();
    assert_eq!(refreshed.backend_service, "sessionrpc-worker-worker-c");
    assert_eq!(refreshed.device_ordinal, 2);
    assert_eq!(refreshed.lease_epoch.get(), 8);
}
