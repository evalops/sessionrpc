# @evalops/sessionrpc

TypeScript client binding for `sessionrpc`.

## What It Includes

- `encodeFrame` and `decodeFrame`, matching the Rust `FrameCodec` wire format.
- `WebRtcFrameTransport`, a browser `RTCDataChannel` transport.
- A small data-channel conformance test using a fake in-process channel pair.

## Usage

```ts
import { FrameKind, WebRtcFrameTransport } from "@evalops/sessionrpc";

const channel = peerConnection.createDataChannel("sessionrpc", {
  ordered: true,
});
const transport = new WebRtcFrameTransport(channel);

await transport.send({
  kind: FrameKind.Data,
  leaseEpoch: 1n,
  payload: new TextEncoder().encode("prompt bytes"),
  seq: 0n,
  sessionId: "972d1893-7fc4-4e48-bd9b-0b99e1868a61",
  streamId: 1n,
  tokenCount: 2n,
});
```

The transport expects ordered, reliable data channels. Signaling, ICE server
selection, auth, and TURN policy belong to the host application.
