use tokio::sync::mpsc;

use crate::{Frame, SessionRpcError};

#[derive(Debug)]
pub struct InMemoryEndpoint {
    tx: mpsc::Sender<Frame>,
    rx: mpsc::Receiver<Frame>,
}

pub fn in_memory_transport_pair(capacity: usize) -> (InMemoryEndpoint, InMemoryEndpoint) {
    let (client_tx, worker_rx) = mpsc::channel(capacity);
    let (worker_tx, client_rx) = mpsc::channel(capacity);

    (
        InMemoryEndpoint {
            tx: client_tx,
            rx: client_rx,
        },
        InMemoryEndpoint {
            tx: worker_tx,
            rx: worker_rx,
        },
    )
}

impl InMemoryEndpoint {
    pub async fn send(&self, frame: Frame) -> Result<(), SessionRpcError> {
        self.tx
            .send(frame)
            .await
            .map_err(|_| SessionRpcError::TransportClosed)
    }

    pub async fn recv(&mut self) -> Result<Frame, SessionRpcError> {
        self.rx.recv().await.ok_or(SessionRpcError::TransportClosed)
    }
}
