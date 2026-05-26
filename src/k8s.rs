use std::collections::HashMap;

use crate::{LeaseEpoch, SessionId, SessionRegistry, SessionSnapshot};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct K8sRoutePolicy {
    pub backend_namespace: String,
    pub service_prefix: String,
    pub backend_port: u16,
    pub sticky_header: String,
}

impl K8sRoutePolicy {
    pub fn new(
        backend_namespace: impl Into<String>,
        service_prefix: impl Into<String>,
        backend_port: u16,
    ) -> Self {
        Self {
            backend_namespace: backend_namespace.into(),
            service_prefix: service_prefix.into(),
            backend_port,
            sticky_header: "x-sessionrpc-session".to_string(),
        }
    }

    pub fn with_sticky_header(mut self, sticky_header: impl Into<String>) -> Self {
        self.sticky_header = sticky_header.into();
        self
    }

    fn backend_service(&self, worker_id: &str) -> String {
        format!("{}-{}", self.service_prefix, dns_label_fragment(worker_id))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StickyRouteDecision {
    pub session_id: SessionId,
    pub sticky_header: String,
    pub sticky_key: String,
    pub backend_namespace: String,
    pub backend_service: String,
    pub backend_port: u16,
    pub worker_id: String,
    pub device_ordinal: u32,
    pub model_id: String,
    pub lease_epoch: LeaseEpoch,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StickyRouteTable {
    routes: HashMap<SessionId, StickyRouteDecision>,
}

impl StickyRouteTable {
    pub fn from_registry(registry: &SessionRegistry, policy: K8sRoutePolicy) -> Self {
        Self::from_snapshots(registry.snapshots(), policy)
    }

    pub fn from_snapshots(
        snapshots: impl IntoIterator<Item = SessionSnapshot>,
        policy: K8sRoutePolicy,
    ) -> Self {
        let routes = snapshots
            .into_iter()
            .map(|snapshot| {
                let route = StickyRouteDecision {
                    session_id: snapshot.session_id,
                    sticky_header: policy.sticky_header.clone(),
                    sticky_key: snapshot.session_id.to_string(),
                    backend_namespace: policy.backend_namespace.clone(),
                    backend_service: policy.backend_service(&snapshot.lease.worker_id),
                    backend_port: policy.backend_port,
                    worker_id: snapshot.lease.worker_id,
                    device_ordinal: snapshot.lease.device_ordinal,
                    model_id: snapshot.lease.model_id,
                    lease_epoch: snapshot.lease_epoch,
                };
                (route.session_id, route)
            })
            .collect();

        Self { routes }
    }

    pub fn route(&self, session_id: SessionId) -> Option<&StickyRouteDecision> {
        self.routes.get(&session_id)
    }

    pub fn snapshot(&self) -> Vec<StickyRouteDecision> {
        let mut routes = self.routes.values().cloned().collect::<Vec<_>>();
        routes.sort_by_key(|route| route.session_id.to_string());
        routes
    }
}

fn dns_label_fragment(value: &str) -> String {
    let mut fragment = String::new();
    let mut last_was_dash = false;

    for ch in value.chars() {
        let next = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            '-'
        };

        if next == '-' {
            if fragment.is_empty() || last_was_dash {
                continue;
            }
            last_was_dash = true;
        } else {
            last_was_dash = false;
        }

        fragment.push(next);
    }

    while fragment.ends_with('-') {
        fragment.pop();
    }

    if fragment.is_empty() {
        "worker".to_string()
    } else {
        fragment
    }
}
