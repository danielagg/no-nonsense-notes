# No Nonsense Notes -- Roadmap

Solo-dev project. Phases are sequential; each ends in something
runnable and testable. Stack details live in [tech-stack.md](tech-stack.md).

## Phase 0 -- Rust Core (local only)

The foundation. No network, no crypto code, no UI.

### Deliverables

- Cargo workspace: `core` crate + `cli` test harness
- **Loro benchmark (gate):** 10k-edit markdown document -- load time,
  memory, update size. Confirms Loro before anything is built on it.
- SQLite schema + numbered migrations from day one (individual `.sql`
  files, auto-discovered by `migration-build` crate — see
  [tech-stack.md](tech-stack.md))
- Note CRUD: one Loro doc per note, stored as blobs in SQLite
- Metadata (folders, tags, note metadata, settings) in dedicated SQLite
  tables, not inside a Loro doc
- FTS5 full-text search over note plaintext
- Markdown parsing (pulldown-cmark) + syntax spans for editor
  highlighting
- Import/export (plain .md files)
- Crypto & key-management **design written down** (shapes the storage
  schema; implementation waits for Phase 2b)
- Sync wire format versioning decided (version field in every payload)
- CI: GitHub Actions, Linux runners -- unit tests, benchmarks

### Exit criteria

- Full note lifecycle (create, edit, search, delete, export) works from
  the CLI harness
- Loro benchmark passes acceptable thresholds
- Core test suite green in CI

## Phase 1 -- Android App (local only)

First real product. Shippable offline app is the milestone.

### Deliverables

- UniFFI Kotlin bindings + NDK build for the core
- Jetpack Compose app shell: note list, folders, tags, search
- **The editor** (the hard part):
  - BasicTextField + visual transformation for markdown styling
  - Live-preview markdown styling in place (no edit/preview split)
  - Incremental restyling -- only the damaged range
  - Tables render properly
  - Checklists toggle by tap
  - Instant typing latency on 10k+ line documents; benchmark on
    mid-range hardware from day one
- Sync staleness indicator: last-synced timestamp, visual cue when
  offline
- Android CI build (Linux runner)

### Exit criteria

- Daily-drivable offline notes app on Android (dogfood on primary
  device)
- Editor latency target holds on large real documents
- Installable release APK (direct install; F-Droid/Play can wait)

## Phase 2a -- Sync Protocol + Server

Two devices, one truth (plaintext over TLS -- encryption comes in 2b).

### Deliverables

- Server: single Rust binary (Axum + Tokio), SQLite storage,
  Litestream backup, behind Caddy (TLS)
- **Server owns its own SQLite schema and migration system** (separate
  from the core crate; same `.sql`-file convention — see
  [tech-stack.md](tech-stack.md))
- Auth: Argon2id -> HKDF split (auth key to server, master key never
  leaves device)
- Email + password signup/signin (no verification, no magic link)
- **Opaque bearer tokens** for session auth (not JWT)
- Change-log protocol: append-only Loro update blobs per document,
  global sequence number, cursor-based pull, WebSocket push
- **Binary wire format**: version byte + message type + payload
- **Flat log table**: global_seq, doc_id, device_id, blob, created_at
- Blobs treated as **opaque bytes** from day one (identical code path
  for 2a plaintext and 2b encrypted)
- API surface (v1): `POST /auth/signup`, `POST /auth/signin`,
  `WS /sync`
- Offline queueing, fast reconnect
- Sync-on-open (+ optional periodic WorkManager refresh); live
  WebSocket while the app is open. No FCM.
- Sync round-trip tests in CI

### Exit criteria

- Android <-> Android sync (phone + emulator or second device --
  one UI stack, isolates sync bugs from UI bugs)
- Concurrent offline edits on both devices merge correctly
- Kill-the-server-mid-sync recovery works

## Phase 2b -- E2E Encryption

Server now stores only ciphertext.

### Deliverables

- E2E encryption: XChaCha20-Poly1305, per-document keys wrapped by
  master key, zeroization
- Device pairing via QR code (x25519 key exchange) -- short design
  pass first
- Encrypted change-log protocol: same protocol as 2a, but blobs are
  now ciphertext
- Tombstone / deletion design: deleted notes eventually purged from
  the server log (design before the protocol freezes)
- Crypto tests in CI -- plaintext != ciphertext asserted,
  decryption round-trip, key derivation reproducibility

### Exit criteria

- Android <-> Android sync with full E2E (phone + emulator)
- Server disk contains only ciphertext (verified, not assumed)

## Phase 3 -- macOS App

UI only; the core is already proven.

### Deliverables

- UniFFI bindings + Swift package for the core
- SwiftUI app: note list, folders, tags, search
- Editor: NSTextView on TextKit 2, wrapped for SwiftUI (SwiftUI
  `TextEditor` is not capable enough); same latency targets
- Persistent WebSocket while running -- sync effectively real-time
- Device pairing flow (scan QR from Android)
- Signed + notarized build (Apple Developer Program -- $99/year cost
  starts here, not before)
- macOS CI build (macOS runner)
- Distribution: Android via F-Droid / direct APK (Play optional)

### Exit criteria

- Android <-> macOS sync round-trip
- Editor latency target holds on large real documents

## v1 = end of Phase 3

Android + macOS, local-first, E2E-encrypted sync, fast markdown
editing. Accepted v1 limitations, stated in the UI:

- Losing all devices (or the password) loses the notes -- no recovery
- No attachments / images
- No sharing or collaboration
- Android sync only while the app is open

## Open questions (not blocking v1)

- **Conflict resolution UX.** CRDT handles data merging automatically,
  but the user may see visual jumps when concurrent edits reconcile.
  Do we show anything, or just let CRDT do its thing silently?
- **Account features.** Password reset, email verification, rate
  limiting, account deletion -- none designed yet.

## Later (unordered, post-v1)

- **iOS** -- cheap after macOS (same SwiftUI + TextKit 2 editor)
- **Recovery phrase** (12/24 words)
- **Web** -- Rust core to WASM (see WASM watch-out in tech-stack.md)
- Sync payload compression (careful: compress-then-encrypt size leaks)
- Attachments, image support
- Shared notebooks, real-time collaborative editing
- Push-based background sync on Android (UnifiedPush / FCM)
- Local at-rest encryption (if lost-device enters the threat model)
- Linux, Windows
