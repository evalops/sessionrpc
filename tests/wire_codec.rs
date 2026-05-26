use bytes::Bytes;
use sessionrpc::{
    Frame, FrameCodec, FrameKind, FrameSeq, LeaseEpoch, SessionId, SessionRpcError, StreamId,
};

#[test]
fn data_frames_roundtrip_through_the_wire_codec() {
    let codec = FrameCodec::default();
    let frame = Frame::data(
        SessionId::new(),
        StreamId::new(42),
        FrameSeq::new(7),
        LeaseEpoch::new(3),
        Bytes::from_static(b"token-delta"),
    );

    let encoded = codec.encode(&frame).unwrap();

    assert_eq!(&encoded[..4], b"SRP1");
    assert_eq!(codec.decode(&encoded).unwrap(), frame);
}

#[test]
fn control_frames_roundtrip_without_payload_credit() {
    let codec = FrameCodec::default();
    let frame = Frame::control(
        SessionId::new(),
        StreamId::new(8),
        FrameSeq::new(0),
        LeaseEpoch::new(9),
        FrameKind::Ping,
    );

    let encoded = codec.encode(&frame).unwrap();

    assert_eq!(codec.decode(&encoded).unwrap(), frame);
}

#[test]
fn decoder_rejects_unsupported_versions_before_routing() {
    let codec = FrameCodec::default();
    let frame = Frame::control(
        SessionId::new(),
        StreamId::new(1),
        FrameSeq::new(0),
        LeaseEpoch::new(1),
        FrameKind::Open,
    );
    let mut encoded = codec.encode(&frame).unwrap().to_vec();
    encoded[4] = 99;

    assert_eq!(
        codec.decode(&encoded).unwrap_err(),
        SessionRpcError::UnsupportedFrameVersion {
            supported: 1,
            actual: 99
        }
    );
}

#[test]
fn decoder_rejects_truncated_payloads() {
    let codec = FrameCodec::default();
    let frame = Frame::data(
        SessionId::new(),
        StreamId::new(1),
        FrameSeq::new(0),
        LeaseEpoch::new(1),
        Bytes::from_static(b"abcdef"),
    );
    let mut encoded = codec.encode(&frame).unwrap().to_vec();
    let needed = encoded.len();
    encoded.truncate(needed - 2);

    assert_eq!(
        codec.decode(&encoded).unwrap_err(),
        SessionRpcError::TruncatedFrame {
            needed,
            actual: needed - 2
        }
    );
}
