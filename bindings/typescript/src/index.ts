const MAGIC = new Uint8Array([0x53, 0x52, 0x50, 0x31]);
const VERSION = 1;
const HEADER_LEN = 60;
const TOKEN_COUNT_NONE = 0xffff_ffff_ffff_ffffn;

export enum FrameKind {
  Data = 0,
  Cancel = 1,
  Open = 2,
  End = 3,
  Ping = 4,
}

export interface SessionRpcFrame {
  sessionId: string;
  streamId: bigint;
  seq: bigint;
  leaseEpoch: bigint;
  kind: FrameKind;
  payload?: Uint8Array;
  tokenCount?: bigint;
  traceContext?: string;
}

export interface DataChannelLike {
  binaryType?: BinaryType;
  readyState?: string;
  addEventListener(type: "message", listener: (event: MessageEventLike) => void): void;
  removeEventListener(type: "message", listener: (event: MessageEventLike) => void): void;
  send(data: ArrayBuffer): void;
}

export interface MessageEventLike {
  data: ArrayBuffer | ArrayBufferView | Blob | string;
}

export class WebRtcFrameTransport {
  private readonly recvQueue: SessionRpcFrame[] = [];
  private readonly waiters: Array<(frame: SessionRpcFrame) => void> = [];
  private readonly onMessage = (event: MessageEventLike): void => {
    void this.decodeMessage(event.data).then((frame) => this.pushFrame(frame));
  };

  constructor(private readonly channel: DataChannelLike) {
    this.channel.binaryType = "arraybuffer";
    this.channel.addEventListener("message", this.onMessage);
  }

  async send(frame: SessionRpcFrame): Promise<void> {
    if (this.channel.readyState && this.channel.readyState !== "open") {
      throw new Error(`RTCDataChannel is ${this.channel.readyState}`);
    }
    const encoded = encodeFrame(frame);
    const packet = encoded.buffer.slice(
      encoded.byteOffset,
      encoded.byteOffset + encoded.byteLength,
    ) as ArrayBuffer;
    this.channel.send(packet);
  }

  async recv(): Promise<SessionRpcFrame> {
    const frame = this.recvQueue.shift();
    if (frame) {
      return frame;
    }
    return new Promise((resolve) => this.waiters.push(resolve));
  }

  close(): void {
    this.channel.removeEventListener("message", this.onMessage);
  }

  private pushFrame(frame: SessionRpcFrame): void {
    const waiter = this.waiters.shift();
    if (waiter) {
      waiter(frame);
      return;
    }
    this.recvQueue.push(frame);
  }

  private async decodeMessage(data: MessageEventLike["data"]): Promise<SessionRpcFrame> {
    if (typeof data === "string") {
      return decodeFrame(new TextEncoder().encode(data));
    }
    if (data instanceof Blob) {
      return decodeFrame(new Uint8Array(await data.arrayBuffer()));
    }
    if (data instanceof ArrayBuffer) {
      return decodeFrame(new Uint8Array(data));
    }
    return decodeFrame(new Uint8Array(data.buffer, data.byteOffset, data.byteLength));
  }
}

export function encodeFrame(frame: SessionRpcFrame): Uint8Array {
  const payload = frame.payload ?? new Uint8Array();
  const trace = frame.traceContext
    ? new TextEncoder().encode(frame.traceContext)
    : new Uint8Array();
  const encoded = new Uint8Array(HEADER_LEN + trace.byteLength + payload.byteLength);
  const view = new DataView(encoded.buffer);

  encoded.set(MAGIC, 0);
  view.setUint8(4, VERSION);
  view.setUint8(5, frame.kind);
  view.setUint32(6, payload.byteLength, false);
  encoded.set(uuidToBytes(frame.sessionId), 10);
  view.setBigUint64(26, frame.streamId, false);
  view.setBigUint64(34, frame.seq, false);
  view.setBigUint64(42, frame.leaseEpoch, false);
  view.setBigUint64(50, frame.tokenCount ?? TOKEN_COUNT_NONE, false);
  view.setUint16(58, trace.byteLength, false);
  encoded.set(trace, 60);
  encoded.set(payload, 60 + trace.byteLength);

  return encoded;
}

export function decodeFrame(encoded: Uint8Array): SessionRpcFrame {
  if (encoded.byteLength < HEADER_LEN) {
    throw new Error(`truncated frame: need ${HEADER_LEN}, got ${encoded.byteLength}`);
  }
  if (!MAGIC.every((byte, index) => encoded[index] === byte)) {
    throw new Error("invalid frame magic");
  }

  const view = new DataView(encoded.buffer, encoded.byteOffset, encoded.byteLength);
  const version = view.getUint8(4);
  if (version !== VERSION) {
    throw new Error(`unsupported frame version ${version}`);
  }

  const payloadLength = view.getUint32(6, false);
  const traceLength = view.getUint16(58, false);
  const needed = HEADER_LEN + traceLength + payloadLength;
  if (encoded.byteLength < needed) {
    throw new Error(`truncated frame: need ${needed}, got ${encoded.byteLength}`);
  }

  const tokenCount = view.getBigUint64(50, false);
  const traceStart = HEADER_LEN;
  const payloadStart = HEADER_LEN + traceLength;
  const frame: SessionRpcFrame = {
    kind: view.getUint8(5) as FrameKind,
    leaseEpoch: view.getBigUint64(42, false),
    payload: encoded.slice(payloadStart, payloadStart + payloadLength),
    seq: view.getBigUint64(34, false),
    sessionId: bytesToUuid(encoded.slice(10, 26)),
    streamId: view.getBigUint64(26, false),
  };

  if (tokenCount !== TOKEN_COUNT_NONE) {
    frame.tokenCount = tokenCount;
  }
  if (traceLength > 0) {
    frame.traceContext = new TextDecoder().decode(encoded.slice(traceStart, traceStart + traceLength));
  }

  return frame;
}

function uuidToBytes(uuid: string): Uint8Array {
  const hex = uuid.replaceAll("-", "");
  if (hex.length !== 32) {
    throw new Error(`invalid UUID ${uuid}`);
  }
  const bytes = new Uint8Array(16);
  for (let i = 0; i < bytes.length; i += 1) {
    bytes[i] = Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

function bytesToUuid(bytes: Uint8Array): string {
  const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
  return [
    hex.slice(0, 8),
    hex.slice(8, 12),
    hex.slice(12, 16),
    hex.slice(16, 20),
    hex.slice(20),
  ].join("-");
}
