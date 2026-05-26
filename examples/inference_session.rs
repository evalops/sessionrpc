use bytes::Bytes;
use sessionrpc::{
    ClientId, FlowController, Frame, FrameSeq, GpuLease, SessionRegistry, SessionRpcError,
    StreamId, in_memory_transport_pair,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), SessionRpcError> {
    let mut registry = SessionRegistry::default();
    let mut flow = FlowController::new(1024);
    let opened = registry.open_session(
        ClientId::new("browser-tab-1"),
        GpuLease::new("worker-a", 0, "llama-70b", 1),
    );
    let (client, mut worker) = in_memory_transport_pair(16);

    let prompt = Frame::data(
        opened.session_id,
        StreamId::new(1),
        FrameSeq::new(0),
        opened.lease_epoch,
        Bytes::from_static(b"write a haiku about vector databases"),
    );
    flow.reserve_frame(&prompt)?;
    registry.accept_frame(prompt.clone())?;
    client.send(prompt).await?;

    let inbound = worker.recv().await?;
    println!(
        "worker={} gpu={} model={} payload={:?}",
        opened.lease.worker_id,
        opened.lease.device_ordinal,
        opened.lease.model_id,
        inbound.payload()
    );

    Ok(())
}
