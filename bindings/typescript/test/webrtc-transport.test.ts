import assert from "node:assert/strict";
import test from "node:test";
import {
  FrameKind,
  WebRtcFrameTransport,
  type SessionRpcFrame,
} from "../src/index.js";
import { FakeDataChannel } from "./support/fake-data-channel.js";

test("WebRtcFrameTransport streams frames bidirectionally over data channels", async () => {
  const [clientChannel, workerChannel] = FakeDataChannel.pair();
  const client = new WebRtcFrameTransport(clientChannel);
  const worker = new WebRtcFrameTransport(workerChannel);
  const clientFrame = frame("client-frame", 1n, 0n);
  const workerFrame = frame("worker-frame", 2n, 0n);

  await client.send(clientFrame);
  await worker.send(workerFrame);

  assert.deepEqual(await worker.recv(), clientFrame);
  assert.deepEqual(await client.recv(), workerFrame);
});

function frame(payload: string, streamId: bigint, seq: bigint): SessionRpcFrame {
  return {
    kind: FrameKind.Data,
    leaseEpoch: 1n,
    payload: new TextEncoder().encode(payload),
    seq,
    sessionId: "972d1893-7fc4-4e48-bd9b-0b99e1868a61",
    streamId,
  };
}
