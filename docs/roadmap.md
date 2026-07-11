# No Nonsense Notes -- Roadmap

Solo-dev project. Phases are sequential; each ends in something
runnable and testable. Stack details live in [tech-stack.md](tech-stack.md).

## Phase 0 -- Rust Core (local only)

The foundation. No network, no crypto code, no UI.

### Deliverables

- Cargo workspace: `core` crate + `cli` test harness
- **Loro benchmark (gate):** 10k-edit markdown document -- load time,
  memory, update size. Confirms Loro before anything is built on it.
- SQLite schema + numbered migrations from day one
- Note CRUD: one Loro doc per note, stored as blobs in SQLite
- Workspace metadata doc (folders, tags, note metadata, settings)
- FTS5 full-text search over note plaintext
- Markdown parsing (pulldown-cmark) + syntax spans for editor
  highlighting
- Import/export (plain .md files)
- Crypto & key-management **design written down** (shapes the storage
  schema; implementation waits for Phase 2)
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
- Android CI build (Linux runner)

### Exit criteria

- Daily-drivable offline notes app on Android (dogfood on primary
  device)
- Editor latency target holds on large real documents
- Installable release APK (direct install; F-Droid/Play can wait)

## Phase 2 -- Sync + E2E Encryption

Two devices, one truth, server sees nothing.

### Deliverables

- Server: single Rust binary (Axum + Tokio), SQLite storage,
  Litestream backup, behind Caddy (TLS)
- Auth: Argon2id -> HKDF split (auth key to server, master key never
  leaves device)
- E2E encryption: XChaCha20-Poly1305, per-document keys wrapped by
  master key, zeroization
- Encrypted change-log protocol: append-only encrypted Loro update
  blobs per document, cursor-based pull, WebSocket push
- Device pairing via QR code (x25519 key exchange) -- short design
  pass first
- Tombstone / deletion design: deleted notes eventually purged from
  the server log (design before the protocol freezes)
- Offline queueing, fast reconnect
- Sync-on-open (+ optional periodic WorkManager refresh); live
  WebSocket while the app is open. No FCM.
- Sync round-trip + crypto tests in CI

### Exit criteria

- Android <-> Android sync (phone + emulator or second device --
  one UI stack, isolates sync bugs from UI bugs)
- Concurrent offline edits on both devices merge correctly
- Server disk contains only ciphertext (verified, not assumed)
- Kill-the-server-mid-sync recovery works

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
