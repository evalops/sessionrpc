use std::{env, net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use sessionrpc::{
    ClientId, GpuLease, K8sRoutePolicy, SessionId, SessionSnapshot, StickyRouteDecision,
    StickyRouteTable,
};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    policy: K8sRoutePolicy,
    routes: Arc<RwLock<StickyRouteTable>>,
}

#[derive(Debug, Deserialize)]
struct SnapshotRequest {
    sessions: Vec<SessionRecord>,
}

#[derive(Debug, Deserialize)]
struct SessionRecord {
    session_id: String,
    client_id: String,
    worker_id: String,
    device_ordinal: u32,
    model_id: String,
    lease_epoch: u64,
}

#[derive(Debug, Serialize)]
struct RouteDecisionResponse {
    session_id: String,
    sticky_header: String,
    sticky_key: String,
    backend_namespace: String,
    backend_service: String,
    backend_port: u16,
    worker_id: String,
    device_ordinal: u32,
    model_id: String,
    lease_epoch: u64,
}

#[tokio::main]
async fn main() {
    let addr = env::var("SESSIONRPC_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse::<SocketAddr>()
        .expect("SESSIONRPC_BIND_ADDR must be host:port");
    let namespace = env::var("SESSIONRPC_BACKEND_NAMESPACE")
        .or_else(|_| env::var("POD_NAMESPACE"))
        .unwrap_or_else(|_| "default".to_string());
    let service_prefix = env::var("SESSIONRPC_WORKER_SERVICE_PREFIX")
        .unwrap_or_else(|_| "sessionrpc-worker".to_string());
    let backend_port = env::var("SESSIONRPC_WORKER_PORT")
        .ok()
        .map(|value| {
            value
                .parse::<u16>()
                .expect("SESSIONRPC_WORKER_PORT must be a u16")
        })
        .unwrap_or(8080);
    let sticky_header =
        env::var("SESSIONRPC_STICKY_HEADER").unwrap_or_else(|_| "x-sessionrpc-session".to_string());

    let policy = K8sRoutePolicy::new(namespace, service_prefix, backend_port)
        .with_sticky_header(sticky_header);
    let state = AppState {
        policy,
        routes: Arc::new(RwLock::new(StickyRouteTable::default())),
    };
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind sticky sidecar listener");

    axum::serve(listener, app(state))
        .await
        .expect("serve sticky sidecar");
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/readyz", get(|| async { StatusCode::NO_CONTENT }))
        .route("/routes", get(list_routes).put(replace_snapshot))
        .route("/routes/{session_id}", get(route_session))
        .with_state(state)
}

async fn replace_snapshot(
    State(state): State<AppState>,
    Json(request): Json<SnapshotRequest>,
) -> Result<StatusCode, SidecarError> {
    let mut snapshots = Vec::with_capacity(request.sessions.len());
    for record in request.sessions {
        snapshots.push(record.into_snapshot()?);
    }

    let table = StickyRouteTable::from_snapshots(snapshots, state.policy.clone());
    *state.routes.write().await = table;

    Ok(StatusCode::NO_CONTENT)
}

async fn route_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<RouteDecisionResponse>, SidecarError> {
    let session_id = parse_session_id(&session_id)?;
    let routes = state.routes.read().await;
    let route = routes
        .route(session_id)
        .cloned()
        .ok_or(SidecarError::UnknownSession)?;

    Ok(Json(RouteDecisionResponse::from(route)))
}

async fn list_routes(State(state): State<AppState>) -> Json<Vec<RouteDecisionResponse>> {
    let routes = state.routes.read().await;
    Json(
        routes
            .snapshot()
            .into_iter()
            .map(RouteDecisionResponse::from)
            .collect(),
    )
}

impl SessionRecord {
    fn into_snapshot(self) -> Result<SessionSnapshot, SidecarError> {
        let session_id = parse_session_id(&self.session_id)?;
        Ok(SessionSnapshot {
            session_id,
            client_id: ClientId::new(self.client_id),
            lease: GpuLease::new(
                self.worker_id,
                self.device_ordinal,
                self.model_id,
                self.lease_epoch,
            ),
            lease_epoch: sessionrpc::LeaseEpoch::new(self.lease_epoch),
        })
    }
}

impl From<StickyRouteDecision> for RouteDecisionResponse {
    fn from(route: StickyRouteDecision) -> Self {
        Self {
            session_id: route.session_id.to_string(),
            sticky_header: route.sticky_header,
            sticky_key: route.sticky_key,
            backend_namespace: route.backend_namespace,
            backend_service: route.backend_service,
            backend_port: route.backend_port,
            worker_id: route.worker_id,
            device_ordinal: route.device_ordinal,
            model_id: route.model_id,
            lease_epoch: route.lease_epoch.get(),
        }
    }
}

fn parse_session_id(value: &str) -> Result<SessionId, SidecarError> {
    let uuid = Uuid::parse_str(value).map_err(|_| SidecarError::BadSessionId)?;
    Ok(SessionId::from_bytes(*uuid.as_bytes()))
}

#[derive(Debug)]
enum SidecarError {
    BadSessionId,
    UnknownSession,
}

impl IntoResponse for SidecarError {
    fn into_response(self) -> Response {
        match self {
            SidecarError::BadSessionId => {
                (StatusCode::BAD_REQUEST, "invalid session_id").into_response()
            }
            SidecarError::UnknownSession => {
                (StatusCode::NOT_FOUND, "unknown session").into_response()
            }
        }
    }
}
