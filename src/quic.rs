use std::{net::SocketAddr, sync::Arc};

use quinn::{
    ClientConfig, Connection, Endpoint, RecvStream, SendStream, ServerConfig, ZeroRttAccepted,
    rustls::{
        RootCertStore,
        pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
    },
};
use rcgen::CertifiedKey;

use crate::{Frame, FrameCodec, FrameTransport, SessionRpcError};

const MAX_FRAME_BYTES: usize = 32 * 1024 * 1024;
const CLOSE_CODE: u32 = 0;

#[derive(Debug)]
pub struct QuicFrameTransport {
    connection: Connection,
    codec: FrameCodec,
    max_frame_bytes: usize,
}

#[derive(Debug)]
pub struct QuicClient {
    endpoint: Endpoint,
    server_addr: SocketAddr,
    server_name: String,
}

pub struct QuicResumeConnection {
    transport: QuicFrameTransport,
    accepted: Option<ZeroRttAccepted>,
}

#[derive(Debug)]
pub struct QuicTestServer {
    endpoint: Endpoint,
    certificate_der: CertificateDer<'static>,
}

impl QuicFrameTransport {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            codec: FrameCodec::default(),
            max_frame_bytes: MAX_FRAME_BYTES,
        }
    }

    pub async fn send(&mut self, frame: Frame) -> Result<(), SessionRpcError> {
        let encoded = self.codec.encode(&frame)?;
        let mut stream = self.connection.open_uni().await.map_err(map_transport)?;
        stream.write_all(&encoded).await.map_err(map_transport)?;
        stream.finish().map_err(map_transport)?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Frame, SessionRpcError> {
        let mut stream = self.connection.accept_uni().await.map_err(map_transport)?;
        let encoded = stream
            .read_to_end(self.max_frame_bytes)
            .await
            .map_err(map_transport)?;
        self.codec.decode(&encoded)
    }

    pub fn close(&mut self) {
        self.connection
            .close(CLOSE_CODE.into(), b"sessionrpc closing");
    }
}

impl FrameTransport for QuicFrameTransport {
    fn send(
        &mut self,
        frame: Frame,
    ) -> impl std::future::Future<Output = Result<(), SessionRpcError>> + Send {
        async move { QuicFrameTransport::send(self, frame).await }
    }

    fn recv(&mut self) -> impl std::future::Future<Output = Result<Frame, SessionRpcError>> + Send {
        async move { QuicFrameTransport::recv(self).await }
    }
}

impl QuicClient {
    pub fn new(
        server_addr: SocketAddr,
        certificate_der: CertificateDer<'static>,
    ) -> Result<Self, SessionRpcError> {
        let mut roots = RootCertStore::empty();
        roots.add(certificate_der).map_err(map_transport)?;
        let mut endpoint = Endpoint::client("127.0.0.1:0".parse().expect("valid bind addr"))
            .map_err(map_transport)?;
        endpoint.set_default_client_config(
            ClientConfig::with_root_certificates(Arc::new(roots)).map_err(map_transport)?,
        );

        Ok(Self {
            endpoint,
            server_addr,
            server_name: "localhost".to_owned(),
        })
    }

    pub async fn connect(&self) -> Result<QuicFrameTransport, SessionRpcError> {
        let connection = self
            .endpoint
            .connect(self.server_addr, &self.server_name)
            .map_err(map_transport)?
            .await
            .map_err(map_transport)?;
        Ok(QuicFrameTransport::new(connection))
    }

    pub async fn connect_0rtt(&self) -> Result<QuicResumeConnection, SessionRpcError> {
        let connecting = self
            .endpoint
            .connect(self.server_addr, &self.server_name)
            .map_err(map_transport)?;

        match connecting.into_0rtt() {
            Ok((connection, accepted)) => Ok(QuicResumeConnection {
                transport: QuicFrameTransport::new(connection),
                accepted: Some(accepted),
            }),
            Err(connecting) => {
                let connection = connecting.await.map_err(map_transport)?;
                Ok(QuicResumeConnection {
                    transport: QuicFrameTransport::new(connection),
                    accepted: None,
                })
            }
        }
    }
}

impl QuicResumeConnection {
    pub fn attempted_0rtt(&self) -> bool {
        self.accepted.is_some()
    }

    pub fn transport_mut(&mut self) -> &mut QuicFrameTransport {
        &mut self.transport
    }

    pub async fn zero_rtt_accepted(&mut self) -> Result<bool, SessionRpcError> {
        match self.accepted.take() {
            Some(accepted) => Ok(accepted.await),
            None => Ok(false),
        }
    }
}

impl QuicTestServer {
    pub async fn bind_local() -> Result<Self, SessionRpcError> {
        let CertifiedKey { cert, signing_key } =
            rcgen::generate_simple_self_signed(vec!["localhost".to_owned()])
                .map_err(map_transport)?;
        let certificate_der = cert.der().clone();
        let private_key = PrivatePkcs8KeyDer::from(signing_key.serialize_der());
        let server_config = ServerConfig::with_single_cert(
            vec![certificate_der.clone()],
            PrivateKeyDer::Pkcs8(private_key),
        )
        .map_err(map_transport)?;
        let endpoint = Endpoint::server(
            server_config,
            "127.0.0.1:0".parse().expect("valid bind addr"),
        )
        .map_err(map_transport)?;

        Ok(Self {
            endpoint,
            certificate_der,
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.endpoint.local_addr().expect("endpoint has local addr")
    }

    pub fn certificate_der(&self) -> CertificateDer<'static> {
        self.certificate_der.clone()
    }

    pub async fn accept(&self) -> Result<QuicFrameTransport, SessionRpcError> {
        let incoming = self
            .endpoint
            .accept()
            .await
            .ok_or(SessionRpcError::TransportClosed)?;
        let connection = incoming.await.map_err(map_transport)?;
        Ok(QuicFrameTransport::new(connection))
    }
}

fn map_transport(error: impl std::fmt::Display) -> SessionRpcError {
    SessionRpcError::Transport(error.to_string())
}

#[allow(dead_code)]
async fn _read_stream(mut stream: RecvStream) -> Result<Vec<u8>, SessionRpcError> {
    stream
        .read_to_end(MAX_FRAME_BYTES)
        .await
        .map_err(map_transport)
}

#[allow(dead_code)]
async fn _write_stream(mut stream: SendStream, bytes: &[u8]) -> Result<(), SessionRpcError> {
    stream.write_all(bytes).await.map_err(map_transport)?;
    stream.finish().map_err(map_transport)
}
