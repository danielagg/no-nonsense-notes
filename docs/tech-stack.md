# No Nonsense Notes -- Tech Stack

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

- SQLite
- SQLite FTS5 for full-text search
- One SQLite database per device
- One Automerge document per note
- Metadata (folders, tags, settings, etc.) stored separately from note documents

## Sync

- **Automerge** (CRDT)
- **Automerge Sync Protocol**
- Authenticated **WebSockets** as the transport
- Delta-based synchronization handled by Automerge
- Rust relay server using a store-and-forward model
- Background sync, batching, and compression
- Fast reconnect and offline queueing

## Shared Core (Rust)

### Core Libraries

- Tokio
- Automerge
- rusqlite
- UniFFI
- pulldown-cmark
- serde
- serde_json
- tracing
- anyhow
- thiserror

### Cryptography (RustCrypto)

- chacha20poly1305
- x25519-dalek
- ed25519-dalek
- argon2
- hkdf
- sha2
- zeroize
- getrandom

### Responsibilities

- Storage
- CRDT integration
- Sync
- Encryption
- Search
- Markdown
- Import/export
- Business logic

## Native Apps

### macOS

- SwiftUI
- Rust via UniFFI

### iOS

- SwiftUI
- Rust via UniFFI

### Android

- Jetpack Compose
- Rust via UniFFI

### Web (Later)

- Rust compiled to WASM
- Thin JavaScript wrapper
- Shared business logic with native apps

## Backend

**Rust**

### Libraries

- Axum
- Tokio
- tokio-tungstenite
- SQLx

### Database

- PostgreSQL (accounts, devices, sync metadata)

### Responsibilities

- Authentication
- Device management
- WebSocket connections
- Relay Automerge sync messages
- Store encrypted updates
- Push updates to connected devices

> The backend never interprets note contents or implements CRDT logic --
> it authenticates clients, persists encrypted updates, and relays
> Automerge sync messages.

## Security

- End-to-end encryption
- Client-side encryption before sync
- Server cannot read note contents
- Multi-device key management
- Secure key derivation (Argon2 + HKDF)
- Memory zeroization for sensitive material

## Markdown

Native note format.

Renderer:

- pulldown-cmark

Supported in v1:

- Headings
- Bold / Italic
- Lists
- Checklists
- Blockquotes
- Tables
- Code blocks
- Links
- Horizontal rules

## Platforms

- macOS
- iOS
- Android
- Web (later)

## Future (Post-v1)

- Attachments
- Image support
- Shared notebooks
- Real-time collaborative editing
- Linux
- Windows

## Architecture

```text
                    SwiftUI              Jetpack Compose            Web (WASM)
                       │                        │                        │
                   UniFFI                  UniFFI                 JS bindings
                       └──────────────┬─────────┴──────────────┐
                                      │
                           Shared Rust Core
    ┌─────────────────────────────────────────────────────────────────────┐
    │ SQLite │ Automerge │ Sync │ Encryption │ Search │ Markdown │ Import │
    └─────────────────────────────────────────────────────────────────────┘
                                      │
                           Automerge Sync Protocol
                                      │
                        Authenticated WebSockets
                                      │
                         Rust Backend (Axum/Tokio)
                                      │
          PostgreSQL (accounts/devices/metadata) + Encrypted Sync Storage
```

## Design Principles

- **SQLite** is the source of truth on every device.
- **Automerge** handles conflict resolution and synchronization.
- **Rust** contains all business logic and is shared across every platform.
- **Native UIs** are thin presentation layers.
- **The backend is intentionally "dumb"**: it authenticates users, stores
  encrypted data, and relays Automerge sync messages without understanding
  document contents. This keeps the system simpler, more secure, and easier
  to maintain.

## Open Questions / Watch-Outs

These aren't stack changes, just items worth deciding on explicitly before
or during v1 build-out:

- **Local at-rest encryption.** The current plan encrypts note content
  before it leaves the device (in transit / on the server), but SQLite on
  disk is plaintext -- which is what lets FTS5 index it directly. Decide
  now whether a lost/stolen device is in scope for your threat model. If
  so, this needs a deliberate design (e.g. OS-level keychain-backed
  encryption at rest) rather than a retrofit later.
- **Multi-device key management.** This is the least-specified part of the
  plan and the hardest problem in it -- how a second device gets the keys
  without the server ever seeing them. Worth a short dedicated design pass
  (QR-code pairing vs. recovery phrase vs. key-transparency log) before
  writing code against it.
- **SQLite + WASM (deferred).** `rusqlite`'s bundled SQLite compiles to
  `wasm32` less cleanly than the pure-Rust crypto crates do -- it needs a
  WASM-capable C toolchain rather than a plain `cargo build`. Solvable
  (prior art: `wa-sqlite`, sql.js), and not a v1 concern since Web is
  scheduled for later, but flagging so it doesn't surprise you.
