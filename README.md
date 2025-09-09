# ws-forwarder

A small Rust WebSocket forwarder/proxy that listens on `localhost:6940` and proxies WebSocket connections to `wss://fstream.binance.com/ws`.

Usage:

1. Build:

    cargo build --release

2. Run:

    ./target/release/ws-forwarder

Then point your client at `ws://localhost:6940` instead of `wss://fstream.binance.com`. This forwarder does not perform TLS for incoming connections — if you need `wss://localhost:6940`, run it behind a TLS terminator (nginx or stunnel) or modify the code to accept TLS.

Notes:
- Minimal proxy; it forwards messages bidirectionally and propagates close frames.
- No authentication or rate limiting included.
# 4RD
