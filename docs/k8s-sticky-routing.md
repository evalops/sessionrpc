# Kubernetes sticky routing

SessionRPC keeps GPU affinity in `SessionRegistry`: each session points at a
worker, GPU ordinal, model, and lease epoch. The k8s sticky-routing layer turns
that registry state into backend decisions that a sidecar, ingress extension, or
proxy can consume without reconstructing affinity from logs.

## Route table

`StickyRouteTable::from_registry` projects live sessions into decisions with:

- the sticky key (`x-sessionrpc-session` by default)
- the backend service and namespace
- worker id, GPU ordinal, model id, and lease epoch

The default service naming policy maps `worker-a` to
`{service_prefix}-worker-a`, so a policy with prefix `sessionrpc-worker` routes
to `sessionrpc-worker-worker-a`.

```rust
use sessionrpc::{K8sRoutePolicy, StickyRouteTable};

let policy = K8sRoutePolicy::new("inference", "sessionrpc-worker", 8080);
let routes = StickyRouteTable::from_registry(&registry, policy);
let decision = routes.route(session_id);
```

## Sidecar example

`examples/k8s_sticky_sidecar.rs` exposes a small HTTP sidecar:

- `PUT /routes` replaces the current route snapshot
- `GET /routes/{session_id}` returns the sticky backend for one session
- `GET /routes` returns all routes
- `GET /readyz` is the readiness probe

Snapshot update body:

```json
{
  "sessions": [
    {
      "session_id": "2f8ad4ce-e85a-4ef9-b274-7c31c4a0b35d",
      "client_id": "browser-tab-1",
      "worker_id": "worker-a",
      "device_ordinal": 0,
      "model_id": "llama-70b",
      "lease_epoch": 7
    }
  ]
}
```

Route response:

```json
{
  "session_id": "2f8ad4ce-e85a-4ef9-b274-7c31c4a0b35d",
  "sticky_header": "x-sessionrpc-session",
  "sticky_key": "2f8ad4ce-e85a-4ef9-b274-7c31c4a0b35d",
  "backend_namespace": "inference",
  "backend_service": "sessionrpc-worker-worker-a",
  "backend_port": 8080,
  "worker_id": "worker-a",
  "device_ordinal": 0,
  "model_id": "llama-70b",
  "lease_epoch": 7
}
```

The deployment sketch in
[`deploy/k8s/sticky-router-sidecar.yaml`](../deploy/k8s/sticky-router-sidecar.yaml)
runs the sticky route sidecar next to a SessionRPC router. The router can push
fresh snapshots after opens, resumes, and lease refreshes; a proxy can then use
the returned backend service to keep reconnecting clients on the same GPU lease.
