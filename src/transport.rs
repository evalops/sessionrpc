use tokio::sync::mpsc;

use bytes::Bytes;

use crate::{Frame, FrameSeq, LeaseEpoch, SessionId, SessionRpcError, StreamId};

pub trait FrameTransport {
    fn send(
        &mut self,
        frame: Frame,
    ) -> impl std::future::Future<Output = Result<(), SessionRpcError>> + Send;

    fn recv(&mut self) -> impl std::future::Future<Output = Result<Frame, SessionRpcError>> + Send;
}

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

impl FrameTransport for InMemoryEndpoint {
    fn send(
        &mut self,
        frame: Frame,
    ) -> impl std::future::Future<Output = Result<(), SessionRpcError>> + Send {
        async move { InMemoryEndpoint::send(self, frame).await }
    }

    fn recv(&mut self) -> impl std::future::Future<Output = Result<Frame, SessionRpcError>> + Send {
        async move { InMemoryEndpoint::recv(self).await }
    }
}

pub async fn assert_transport_conformance<T, Make>(make_pair: Make) -> Result<(), SessionRpcError>
where
    T: FrameTransport,
    Make: FnOnce() -> (T, T),
{
    let (client, worker) = make_pair();
    assert_transport_conformance_pair(client, worker).await
}

pub async fn assert_transport_conformance_pair<T>(
    mut client: T,
    mut worker: T,
) -> Result<(), SessionRpcError>
where
    T: FrameTransport,
{
    let session_id = SessionId::new();
    let lease_epoch = LeaseEpoch::new(1);
    let client_first = Frame::data(
        session_id,
        StreamId::new(1),
        FrameSeq::new(0),
        lease_epoch,
        Bytes::from_static(b"client-first"),
    );
    let client_second = Frame::data(
        session_id,
        StreamId::new(1),
        FrameSeq::new(1),
        lease_epoch,
        Bytes::from_static(b"client-second"),
    );
    let worker_reply = Frame::data(
        session_id,
        StreamId::new(2),
        FrameSeq::new(0),
        lease_epoch,
        Bytes::from_static(b"worker-reply"),
    );

    client.send(client_first.clone()).await?;
    client.send(client_second.clone()).await?;
    worker.send(worker_reply.clone()).await?;

    assert_eq!(worker.recv().await?, client_first);
    assert_eq!(worker.recv().await?, client_second);
    assert_eq!(client.recv().await?, worker_reply);

    Ok(())
}
