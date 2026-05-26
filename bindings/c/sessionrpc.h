#ifndef SESSIONRPC_H
#define SESSIONRPC_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

enum {
  SESSIONRPC_FRAME_DATA = 0,
  SESSIONRPC_FRAME_CANCEL = 1,
  SESSIONRPC_FRAME_OPEN = 2,
  SESSIONRPC_FRAME_END = 3,
  SESSIONRPC_FRAME_PING = 4,
};

typedef enum SessionRpcStatus {
  SessionRpcStatus_Ok = 0,
  SessionRpcStatus_InvalidArgument = 1,
  SessionRpcStatus_DecodeError = 2,
} SessionRpcStatus;

typedef struct SessionRpcBytes {
  const uint8_t *ptr;
  size_t len;
} SessionRpcBytes;

typedef struct SessionRpcBuffer {
  uint8_t *ptr;
  size_t len;
  size_t capacity;
} SessionRpcBuffer;

typedef struct SessionRpcFrameView {
  uint8_t session_id[16];
  uint64_t stream_id;
  uint64_t seq;
  uint64_t lease_epoch;
  uint8_t kind;
  SessionRpcBytes payload;
  bool has_token_count;
  uint64_t token_count;
  SessionRpcBytes trace_context;
} SessionRpcFrameView;

typedef struct SessionRpcDecodedFrame {
  uint8_t session_id[16];
  uint64_t stream_id;
  uint64_t seq;
  uint64_t lease_epoch;
  uint8_t kind;
  SessionRpcBytes payload;
  size_t payload_capacity;
  bool has_token_count;
  uint64_t token_count;
  SessionRpcBytes trace_context;
  size_t trace_context_capacity;
} SessionRpcDecodedFrame;

SessionRpcStatus sessionrpc_encode_frame(const SessionRpcFrameView *frame,
                                         SessionRpcBuffer *out);

SessionRpcStatus sessionrpc_decode_frame(const uint8_t *encoded,
                                         size_t encoded_len,
                                         SessionRpcDecodedFrame *out);

void sessionrpc_buffer_free(SessionRpcBuffer *buffer);

void sessionrpc_decoded_frame_free(SessionRpcDecodedFrame *frame);

#ifdef __cplusplus
}
#endif

#endif
