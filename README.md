# No Nonsense Notes

Local-first, E2E-encrypted markdown and list notes. CRDT sync. Fast above everything.

Solo-dev project.

## Status

| Component | Status |
|---|---|
| Rust core (note CRUD, FTS5, schema) | In progress (Phase 0) |
| Server (sync, auth) | Stub (`println!("not yet implemented")`) |
| Android app | Not started (Phase 1) |
| macOS app | Not started (Phase 3) |
| iOS app | Not started (post-v1) |
| Web app | Not started (post-v1) |

## Architecture

```text
              Jetpack Compose          SwiftUI + TextKit 2         Web (WASM)
                       │                        │                        │
                   UniFFI                   UniFFI                JS bindings
                       └──────────────┬─────────┴──────────────┐
                                      │
                           Shared Rust Core
    ┌─────────────────────────────────────────────────────────────────────┐
    │  SQLite │ Loro │ Sync │ Encryption │ Search │ Markdown │ Import     │
    └─────────────────────────────────────────────────────────────────────┘
                                      │
                    Encrypted change-log protocol (own, thin)
                                      │
                        Authenticated WebSockets (via Caddy/TLS)
                                      │
                         Rust Backend (Axum/Tokio)
                                      │
              SQLite (accounts / devices / encrypted update logs)
```

## Stack

- **Core:** Rust — Loro (CRDT), rusqlite, UniFFI, pulldown-cmark, RustCrypto
- **Server:** Axum + Tokio, SQLite, Caddy (TLS)
- **Android:** Jetpack Compose + Kotlin via UniFFI
- **macOS/iOS:** SwiftUI + TextKit 2 via UniFFI
- **Web:** Rust → WASM, thin JS wrapper (post-v1)

## Project structure

```text
no-nonsense-notes/
├── crates/
│   ├── core/           Shared Rust library (storage, CRDT, sync, crypto, search)
│   └── server/         Sync server binary
├── apps/
│   ├── android/        Jetpack Compose app (Phase 1)
│   ├── macos/          SwiftUI app (Phase 3)
│   ├── ios/            SwiftUI app (post-v1)
│   └── web/            WASM app (post-v1)
├── features/           BDD scenarios (Gherkin)
├── scripts/            Build/CI helpers
└── docs/               Design docs (tech stack, security, sync, roadmap)
```

## Building

```bash
# Core library
cargo build -p no-nonsense-notes-core

# Run core tests
cargo test -p no-nonsense-notes-core

# Server (not yet implemented)
cargo run -p no-nonsense-notes-server
```

## Design principles

- **SQLite** is the source of truth on every device
- **Loro** handles conflict resolution; sync moves encrypted updates around
- **Rust** contains all business logic, shared across every platform
- **Native UIs** are thin presentation layers
- **The backend is intentionally dumb**: authenticates, appends, streams. Never reads content.
- **Minimal dependencies, minimal complexity**

## Docs

| Doc | What's in it |
|---|---|
| [CONTEXT.md](CONTEXT.md) | Domain glossary and resolved decisions |
| [tech-stack.md](docs/tech-stack.md) | Stack overview, architecture, threading model |
| [roadmap.md](docs/roadmap.md) | Phases 0–3, deliverables, exit criteria |
| [security.md](docs/security.md) | E2E encryption, auth, key derivation, device pairing |
| [sync.md](docs/sync.md) | Loro CRDT, encrypted change-log protocol, transport |
| [editor.md](docs/editor.md) | Live-preview markdown editor, per-platform approach |
| [testing.md](docs/testing.md) | Test categories, benchmarks, CI layout |

## License

MIT
