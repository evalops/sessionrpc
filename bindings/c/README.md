# SessionRPC C binding

The C binding exposes the stable wire-frame boundary for C and C-compatible
runtimes. Build the Rust crate as a static or dynamic library:

```bash
cargo build --release
```

Artifacts are emitted under `target/release`:

- `libsessionrpc.a`
- `libsessionrpc.dylib` on macOS, or the platform dynamic-library equivalent

Include [`sessionrpc.h`](sessionrpc.h) and link against the generated library.

## Memory contract

`sessionrpc_encode_frame` allocates `SessionRpcBuffer.ptr`. Call
`sessionrpc_buffer_free` exactly once when the encoded bytes are no longer
needed.

`sessionrpc_decode_frame` allocates payload and trace buffers inside
`SessionRpcDecodedFrame`. Call `sessionrpc_decoded_frame_free` exactly once when
the decoded view is no longer needed.

Inputs are borrowed only for the duration of the call.

## Minimal encode flow

```c
#include "sessionrpc.h"

uint8_t session_id[16] = {0};
uint8_t payload[] = "prompt bytes";

SessionRpcFrameView frame = {
    .session_id = {0},
    .stream_id = 1,
    .seq = 0,
    .lease_epoch = 7,
    .kind = SESSIONRPC_FRAME_DATA,
    .payload = {.ptr = payload, .len = sizeof(payload) - 1},
    .has_token_count = false,
    .token_count = 0,
    .trace_context = {.ptr = 0, .len = 0},
};

for (size_t i = 0; i < 16; i++) {
  frame.session_id[i] = session_id[i];
}

SessionRpcBuffer encoded = {0};
if (sessionrpc_encode_frame(&frame, &encoded) == SessionRpcStatus_Ok) {
  /* send encoded.ptr[0..encoded.len] */
  sessionrpc_buffer_free(&encoded);
}
```
