from __future__ import annotations

import struct
from dataclasses import dataclass
from enum import IntEnum
from typing import Optional, Union
from uuid import UUID

MAGIC = b"SRP1"
VERSION = 1
HEADER_LEN = 60
TOKEN_COUNT_NONE = (1 << 64) - 1
HEADER = struct.Struct(">4sBBI16sQQQQH")


class FrameKind(IntEnum):
    DATA = 0
    CANCEL = 1
    OPEN = 2
    END = 3
    PING = 4


@dataclass(frozen=True)
class SessionRpcFrame:
    session_id: UUID
    stream_id: int
    seq: int
    lease_epoch: int
    kind: FrameKind
    payload: bytes = b""
    token_count: Optional[int] = None
    trace_context: Optional[str] = None


def encode_frame(frame: SessionRpcFrame) -> bytes:
    payload = bytes(frame.payload) if frame.kind == FrameKind.DATA else b""
    trace_context = (
        frame.trace_context.encode("utf-8") if frame.trace_context is not None else b""
    )
    if len(payload) > 0xFFFFFFFF:
        raise ValueError(f"frame payload is too large: {len(payload)} bytes")
    if len(trace_context) > 0xFFFF:
        raise ValueError(
            f"frame trace context is too large: {len(trace_context)} bytes"
        )
    token_count = (
        TOKEN_COUNT_NONE if frame.token_count is None else _check_u64(frame.token_count)
    )

    return b"".join(
        [
            HEADER.pack(
                MAGIC,
                VERSION,
                int(frame.kind),
                len(payload),
                frame.session_id.bytes,
                _check_u64(frame.stream_id),
                _check_u64(frame.seq),
                _check_u64(frame.lease_epoch),
                token_count,
                len(trace_context),
            ),
            trace_context,
            payload,
        ]
    )


def decode_frame(encoded: Union[bytes, bytearray, memoryview]) -> SessionRpcFrame:
    encoded = bytes(encoded)
    if len(encoded) < HEADER_LEN:
        raise ValueError(f"truncated frame: need {HEADER_LEN}, got {len(encoded)}")

    (
        magic,
        version,
        kind_value,
        payload_len,
        session_bytes,
        stream_id,
        seq,
        lease_epoch,
        token_count,
        trace_len,
    ) = HEADER.unpack_from(encoded)

    if magic != MAGIC:
        raise ValueError("invalid frame magic")
    if version != VERSION:
        raise ValueError(f"unsupported frame version {version}")

    try:
        kind = FrameKind(kind_value)
    except ValueError as exc:
        raise ValueError(f"unknown frame kind {kind_value}") from exc

    needed = HEADER_LEN + trace_len + payload_len
    if len(encoded) < needed:
        raise ValueError(f"truncated frame: need {needed}, got {len(encoded)}")

    trace_start = HEADER_LEN
    payload_start = trace_start + trace_len
    trace_context = (
        encoded[trace_start:payload_start].decode("utf-8") if trace_len else None
    )
    payload = encoded[payload_start:needed] if kind == FrameKind.DATA else b""

    return SessionRpcFrame(
        session_id=UUID(bytes=session_bytes),
        stream_id=stream_id,
        seq=seq,
        lease_epoch=lease_epoch,
        kind=kind,
        payload=payload,
        token_count=None if token_count == TOKEN_COUNT_NONE else token_count,
        trace_context=trace_context,
    )


def _check_u64(value: int) -> int:
    if value < 0 or value > TOKEN_COUNT_NONE:
        raise ValueError(f"value is outside u64 range: {value}")
    return value


__all__ = [
    "FrameKind",
    "SessionRpcFrame",
    "decode_frame",
    "encode_frame",
]
