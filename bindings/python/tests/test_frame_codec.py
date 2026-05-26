import unittest
from uuid import UUID

from sessionrpc import FrameKind, SessionRpcFrame, decode_frame, encode_frame


class FrameCodecTests(unittest.TestCase):
    def test_roundtrips_payload_tokens_and_trace_context(self) -> None:
        frame = SessionRpcFrame(
            session_id=UUID("2f8ad4ce-e85a-4ef9-b274-7c31c4a0b35d"),
            stream_id=9,
            seq=3,
            lease_epoch=11,
            kind=FrameKind.DATA,
            payload=b"hello from python",
            token_count=5,
            trace_context=(
                "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00"
            ),
        )

        encoded = encode_frame(frame)
        decoded = decode_frame(encoded)

        self.assertEqual(decoded, frame)

    def test_rejects_truncated_frames(self) -> None:
        with self.assertRaisesRegex(ValueError, "truncated frame"):
            decode_frame(b"SRP1")


if __name__ == "__main__":
    unittest.main()
