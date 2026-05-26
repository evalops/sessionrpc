# SessionRPC Python binding

The Python binding is dependency-free and mirrors the Rust wire-frame codec.
It is useful for Python clients, integration tests, and protocol fixtures.

```python
from uuid import UUID

from sessionrpc import FrameKind, SessionRpcFrame, encode_frame, decode_frame

frame = SessionRpcFrame(
    session_id=UUID("2f8ad4ce-e85a-4ef9-b274-7c31c4a0b35d"),
    stream_id=1,
    seq=0,
    lease_epoch=7,
    kind=FrameKind.DATA,
    payload=b"prompt bytes",
    token_count=12,
)

encoded = encode_frame(frame)
decoded = decode_frame(encoded)
assert decoded == frame
```

Run the tests:

```bash
PYTHONPATH=. python -m unittest discover -s tests
```
