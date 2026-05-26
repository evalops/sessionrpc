import assert from "node:assert/strict";
import test from "node:test";
import {
  FrameKind,
  decodeFrame,
  encodeFrame,
  type SessionRpcFrame,
} from "../src/index.js";

test("frame codec roundtrips payload, tokens, and trace context", () => {
  const frame: SessionRpcFrame = {
    kind: FrameKind.Data,
    leaseEpoch: 3n,
    payload: new TextEncoder().encode("token-delta"),
    seq: 7n,
    sessionId: "972d1893-7fc4-4e48-bd9b-0b99e1868a61",
    streamId: 42n,
    tokenCount: 2n,
    traceContext: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
  };

  const encoded = encodeFrame(frame);

  assert.equal(new TextDecoder().decode(encoded.subarray(0, 4)), "SRP1");
  assert.deepEqual(decodeFrame(encoded), frame);
});
