type Listener = (event: { data: ArrayBuffer }) => void;

export class FakeDataChannel {
  readonly readyState = "open";
  private listeners = new Set<Listener>();
  private peer?: FakeDataChannel;

  static pair(): [FakeDataChannel, FakeDataChannel] {
    const a = new FakeDataChannel();
    const b = new FakeDataChannel();
    a.peer = b;
    b.peer = a;
    return [a, b];
  }

  addEventListener(type: string, listener: Listener): void {
    if (type === "message") {
      this.listeners.add(listener);
    }
  }

  removeEventListener(type: string, listener: Listener): void {
    if (type === "message") {
      this.listeners.delete(listener);
    }
  }

  send(data: ArrayBuffer | ArrayBufferView): void {
    const bytes =
      data instanceof ArrayBuffer
        ? new Uint8Array(data)
        : new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    const copy = bytes.slice();
    queueMicrotask(() => {
      for (const listener of this.peer?.listeners ?? []) {
        listener({ data: copy.buffer });
      }
    });
  }
}
