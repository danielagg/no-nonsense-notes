# No Nonsense Notes

Local-first, E2E-encrypted markdown and list notes. CRDT sync. Fast above everything.

Solo-dev project.

## Status

| Component | Status |
|---|---|
| Rust core (note CRUD, FTS5, schema) | In progress (Phase 0) |
| Server (sync, auth, Swagger UI) | In progress |
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
# Start everything (server + web dev server) from project root
./scripts/dev.sh

# Or run individually:
cargo build -p no-nonsense-notes-core
cargo test -p no-nonsense-notes-core
cargo run -p no-nonsense-notes-server

# Web sandbox (apps/web)
cd apps/web && npm install && npm run dev
```

`scripts/dev.sh` launches both the Rust server on `:3000` and the Vite dev server on `:5173` in one command. Ctrl-C kills both.

### Environment variables

| File | Loaded by | Purpose |
|---|---|---|
| `.env.local` (project root) | Rust server (`dotenvy`) | `CORS_ORIGIN`, `DATABASE_URL`, etc. |
| `apps/web/.env.local` | Vite | `VITE_API_URL` |

Both are gitignored. Copy `.env` → `.env.local` and fill in values for your machine.

## API Documentation

The server ships with interactive API docs via Swagger UI.

1. Start the server: `RUST_LOG=info cargo run -p no-nonsense-notes-server`
2. Open **http://localhost:3000/swagger-ui** in your browser
3. Raw OpenAPI spec: **http://localhost:3000/api-docs/openapi.json**

### CORS

The server reads `CORS_ORIGIN` to set the allowed origin. Locally, `.env.local` at the project root is loaded automatically via `dotenvy`:

```
# .env.local (gitignored)
CORS_ORIGIN=http://localhost:5173
```

On Render, set `CORS_ORIGIN` as an environment variable to your Vercel frontend URL.

### Quick test

```bash
# Sign up
curl -s -X POST http://localhost:3000/auth/signup \
  -H 'Content-Type: application/json' \
  -d '{"email":"test@test.com","password":"secret123"}'

# Sign in
curl -s -X POST http://localhost:3000/auth/signin \
  -H 'Content-Type: application/json' \
  -d '{"email":"test@test.com","password":"secret123"}'
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
