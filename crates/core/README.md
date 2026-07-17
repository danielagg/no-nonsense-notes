# Core crate

`no-nonsense-notes-core` contains the platform-independent application logic.
Android, the WASM bridge, and future Apple clients build on this crate.

## Responsibilities

- Note and checklist models backed by Loro documents
- SQLite and in-memory storage implementations
- Full-text and in-memory search
- Native sync outbox, pull application, and WebSocket client
- Shared sync wire-format encoding and decoding
- SQLite schema migrations

The default `sqlite` feature enables native SQLite storage and the blocking
WebSocket client. The `wasm` feature selects browser-compatible UUID and time
support without SQLite.

## Commands

From the repository root:

```sh
cargo check -p no-nonsense-notes-core
cargo test -p no-nonsense-notes-core
```

Add client migrations as numbered SQL files under
`src/storage/migrations/`. The build script discovers them automatically using
[`migration-build`](../migration-build/README.md).

See [the sync design](../../docs/sync.md) and
[testing strategy](../../docs/testing.md) for the larger contracts.
