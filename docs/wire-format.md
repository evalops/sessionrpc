# Wire Format

`FrameCodec` encodes each `Frame` as a fixed header followed by an optional data
payload. Integers are big-endian.

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 4 | Magic bytes: `SRP1` |
| 4 | 1 | Version: `1` |
| 5 | 1 | Frame kind |
| 6 | 4 | Payload length in bytes |
| 10 | 16 | Session id UUID bytes |
| 26 | 8 | Stream id |
| 34 | 8 | Frame sequence |
| 42 | 8 | Lease epoch |
| 50 | 8 | Token count, or `u64::MAX` when absent |
| 58 | N | Payload bytes |

## Frame Kinds

| Value | Kind |
| ---: | --- |
| 0 | Data |
| 1 | Cancel |
| 2 | Open |
| 3 | End |
| 4 | Ping |

## Decode Failures

The decoder rejects invalid magic bytes, unsupported versions, unknown frame
kinds, truncated frames, and payload lengths above the configured maximum. This
lets network transports fail malformed input before it reaches routing or GPU
dispatch.
