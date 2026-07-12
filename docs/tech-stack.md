# No Nonsense Notes -- Tech Stack

Overview document. Deep dives live in their own files:

- [roadmap.md](roadmap.md) -- phases, deliverables, exit criteria
- [security.md](security.md) -- E2E encryption, auth, key management
- [sync.md](sync.md) -- CRDT (Loro), sync protocol, tombstones,
  migrations
- [editor.md](editor.md) -- editor architecture, markdown support
- [testing.md](testing.md) -- test categories, benchmarks, CI layout

## Philosophy

- Local-first
- Offline-first
- Native UI
- Performance above everything
- Open-source where practical
- One shared Rust core across all platforms
- End-to-end encrypted by design
- Minimal dependencies, minimal complexity

## Client Storage

- SQLite -- the source of truth on every device
- SQLite FTS5 for full-text search
- One SQLite database per device
- One Loro document per note (stored as blobs in SQLite)
- Folders, tags, note metadata, settings in dedicated SQLite tables
  (not inside a Loro doc -- avoids decryption overhead for listing)
- Details: [sync.md](sync.md)

## Shared Core (Rust)

### Core Libraries

- loro
- rusqlite
- UniFFI
- tungstenite (blocking WebSocket client on a dedicated thread --
  no async runtime in the client core; Tokio lives server-side only)
- pulldown-cmark (preview rendering, export, and syntax spans for
  editor highlighting)
- serde
- serde_json (scoped: import/export and config only -- the sync wire
  format is raw encrypted bytes, no JSON)
- tracing
- thiserror (typed errors; also required for UniFFI error enums
  across the FFI boundary)
- zeroize
- RustCrypto crates: see [security.md](security.md)

### Responsibilities

- Storage
- CRDT integration
- Sync
- Encryption
- Search
- Markdown
- Import/export
- Business logic

## Threading model

The Rust core is synchronous; the WebSocket runs on a dedicated
`std::thread`. When updates arrive, Rust calls back into the native
layer via a **UniFFI callback interface** (`SyncDelegate` trait).
The native side implements the trait and dispatches to its UI thread:

- **Android:** callback posts to `Dispatchers.Main`
- **macOS:** callback dispatches to `DispatchQueue.main.async`

This keeps the WebSocket thread isolated from the UI layer. The
callback interface is defined in the UniFFI UDL and generated for
both Kotlin and Swift bindings. No async runtime leaks into the
client core -- Tokio stays server-side only.

## Native Apps

Thin presentation layers over the Rust core. Editor details:
[editor.md](editor.md).

### Android (Phase 1)

- Jetpack Compose
- Rust via UniFFI (Kotlin bindings, NDK build)

### macOS (Phase 3)

- SwiftUI (chrome) + TextKit 2 editor
- Rust via UniFFI (Swift package)

### iOS (Later)

- SwiftUI (chrome) + TextKit 2 editor -- cheap after macOS

### Web (Later)

- Rust compiled to WASM
- Thin JavaScript wrapper
- Shared business logic with native apps

## Backend

**Rust** -- one single-binary process on a cheap VPS. Intentionally
"dumb": it authenticates clients, appends encrypted blobs, and streams
them to devices without understanding document contents.

### Libraries

- Axum
- Tokio
- tokio-tungstenite
- rusqlite (same crate as the client core)
- anyhow (application-level error handling; the core library uses
  thiserror)

### Database

- **SQLite** (accounts, devices, sync metadata, encrypted update logs)
- Continuous backup via Litestream (or equivalent WAL streaming)
- No PostgreSQL: the server only stores accounts and opaque encrypted
  blobs; SQLite handles this scale for years, zero DB ops for a solo dev

### TLS / Ingress

- **Caddy** reverse proxy in front of Axum -- automatic Let's Encrypt
  certificates, zero TLS code

### Auth

- Email + password signup/signin (no verification, no magic link in v1)
- **Opaque bearer tokens** (not JWT) — simple random tokens stored in
  the `auth_tokens` table, verified on each request
- Password hashed with Argon2id server-side (the auth key from the
  client's HKDF derivation)

### Schema ownership & migrations

- **Server owns its own SQLite schema and migration system**, separate
  from the core crate's migrations.
- Both crates follow the **same convention** for schema migrations:
  - Migrations live in `src/storage/migrations/` as individual `.sql`
    files named `NNN_description.sql` (e.g. `001_initial.sql`,
    `002_add_note_type.sql`).
  - The numeric prefix is the version number; the remainder becomes the
    migration description (underscores → spaces).
  - A `build.rs` calls `migration_build::generate()` (shared crate in
    `crates/migration-build/`) which scans the directory at compile
    time and generates a `MIGRATIONS` static array via `include_str!`.
  - **To add a migration:** drop a new `.sql` file into the
    `migrations/` directory. No other code changes needed — the build
    script picks it up automatically.
  - Each migration is tracked in a `_schema_version` table with
    `version`, `description`, and `applied_at` columns.
  - Migrations are idempotent — safe to run multiple times.

### Responsibilities

- Authentication
- Device management
- WebSocket connections
- Persist encrypted update logs (append-only, per document)
- Push new updates to connected devices

## Phases

See [roadmap.md](roadmap.md). Short version: Phase 0 Rust core (local
only) -> Phase 1 Android app (local only) -> Phase 2a sync protocol ->
Phase 2b E2E encryption -> Phase 3 macOS = v1. iOS and Web later.

## Costs (accepted)

- Apple Developer Program: $99/year (required for notarized
  distribution; cost starts in Phase 3, not before)
- One small VPS for the sync server
- Everything else: free and open source. Android distributed via
  F-Droid / direct APK (Play optional).

## Tooling / CI

- GitHub Actions (free tier)
- Rust core is ~90% of the logic and tests on Linux runners: unit
  tests, sync round-trip tests, crypto tests, benchmarks
- Platform UI builds: Linux runner for Android, macOS runner for Swift

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

## Design Principles

- **SQLite** is the source of truth on every device.
- **Loro** handles conflict resolution; sync is just moving encrypted
  Loro updates around.
- **Rust** contains all business logic and is shared across every
  platform.
- **Native UIs** are thin presentation layers.
- **The backend is intentionally "dumb"**: simpler, more secure,
  easier to maintain.

## Watch-Outs

Topic-specific open questions live in their files
([security.md](security.md), [sync.md](sync.md)). One cross-cutting
item stays here:

- **SQLite + WASM (deferred).** `rusqlite`'s bundled SQLite compiles to
  `wasm32` less cleanly than the pure-Rust crypto crates do -- needs a
  WASM-capable C toolchain. Solvable (prior art: `wa-sqlite`, sql.js);
  not a v1 concern since Web is later.
