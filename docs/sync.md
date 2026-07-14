# CRDT & Sync

How notes stay consistent across devices without the server ever
reading them. Encryption details in [security.md](security.md).

## CRDT: Loro

- **Loro** (Rust-native CRDT), one document per note
- Metadata (folders, tags, settings, and most note metadata) is stored in
  dedicated SQLite tables. The user-owned title is stored in the Loro document
  at `metadata.title` so it uses the same CRDT sync as note content, and is
  mirrored in SQLite for fast listing and FTS.
- Metadata changes sync via the same protocol (encrypted version-vector
  over the same WebSocket), not a separate mechanism
- Chosen over Automerge: faster, and shallow snapshots solve the
  unbounded-history-growth problem for long-lived, heavily edited notes
- **Phase 0 gate:** benchmark a 10k-edit markdown document (load time,
  memory, update size) before building on top of it
- Documents stored as blobs in SQLite; FTS5 indexes extracted plaintext

## Sync protocol: encrypted change-log

Own thin protocol. The stock Loro sync protocol requires both
peers to read document state (heads, bloom filters), which is
incompatible with a blind relay -- hence this design:

- Server keeps an **append-only log of Loro update blobs**
  with a single global monotonic sequence number per entry
- Client tracks `last_seen_global_seq`; pull requests "give me
  everything after N" -- simpler than per-doc cursors and avoids
  missed-document edge cases
- Response is a list of `(doc_id, blob, global_seq)` pairs;
  client imports each blob into its local Loro doc
- Client push: export local updates -> append to the server log
- The server never participates in CRDT logic and treats all blobs
  as **opaque bytes** (even in Phase 2a before encryption ships;
  identical code path for 2a and 2b)
- Wire format: **binary protocol** (no JSON), with a version byte +
  message type + payload in every frame
- Server SQLite: **flat log table** (`global_seq`, `doc_id`,
  `device_id`, `blob`, `created_at`) -- simple append-only, client
  pulls everything after its cursor

## Transport & behavior

- Authenticated **WebSockets** (instant push to connected devices),
  TLS via Caddy
- After a push is committed, the server sends an account-scoped
  `update:<global_seq>` notification to every connected session. The
  notification is only a wake-up signal; each client still pulls from its
  own durable cursor, so reconnects and missed notifications are safe.
- Offline queueing, fast reconnect
- **Android:** sync-on-open (+ optional periodic WorkManager refresh);
  live WebSocket while the app is open. No FCM -- keeps the app
  Google-free and F-Droid-friendly. Push-based background sync
  (UnifiedPush / FCM) is post-v1.
- **macOS:** persistent WebSocket while running; sync is effectively
  real-time

## Architecture: shared protocol, platform-specific transport

One sync mechanism. The protocol (wire format encode/decode) and merge
logic live in the Rust core. Only the WebSocket transport differs by
platform — WASM has no threads or blocking I/O, so it uses
`web-sys::WebSocket` (async, callback-based). Native apps use
`tungstenite` on a dedicated `std::thread`.

```
crates/core/src/sync/
  protocol.rs    — wire format encode/decode (pure, works on WASM + native)
  client.rs      — native client (std::thread + tungstenite) [stub]

crates/wasm/src/lib.rs
  — WASM bindings: encodePushFrame, decodePushResponse,
    encodePullRequest, decodePullResponse, applyRemoteUpdate, applyRemoteDelete,
    getSyncCursor, setSyncCursor, getDeviceId, exportNoteBlob

apps/web/src/hooks/use-sync.ts
  — thin: WebSocket transport calls Rust for all framing/merge
```

### Sync blob format

The server treats blobs as opaque bytes. Inside each blob:

```
[kind:1][payload:N]
```

- `kind`: 0=markdown, 1=list, 255=deletion tombstone
  (the note kinds enable receivers to extract content
  without guessing the Loro container)
- `payload`: Loro snapshot/update bytes for a note; tombstones have no payload.
  Note documents contain their user-owned title at `metadata.title`; content
  and list-item edits never derive or replace it.

The server never inspects this — it stores and relays the blob as-is.

## Current implementation status

### Server (done)

- `crates/server/src/sync.rs` -- WebSocket endpoint, binary push,
  text-based pull, auth token verification
- Append-only `updates` table in server SQLite
- Binary wire format: `[version:1][type:1][doc_id:16][device_id:16][blob_len:4][blob:N]`
- Pull response: text `seq:N\ndoc_id:base64_blob\n...`

### Core protocol (done)

- `crates/core/src/sync/protocol.rs` -- full encode/decode:
  `encode_push_frame`, `encode_delete_frame`, `decode_push_response`, `encode_pull_request`,
  `decode_pull_response`, `encode_sync_blob`, `decode_sync_blob`
- `crates/core/src/storage/memory.rs` -- `apply_remote_update`: merges
  remote Loro blobs into existing notes or creates new notes from
  remote; `apply_remote_delete` idempotently applies tombstones

### Web sync (done)

- `apps/web/src/hooks/use-sync.ts` -- WebSocket transport only;
  all protocol encode/decode and merge logic calls into Rust via WASM
- `apps/web/src/lib/sync-manager.ts` -- bridges mutations to push:
  `api.ts` calls `pushNote` after each local mutation
- `apps/web/src/lib/wasm.ts` -- exposes note and tombstone framing,
  `decodePullResponse`, remote update/delete application, sync cursor, device ID
- Flow: local mutation → `pushNote` → `encodePushFrame` (Rust) →
  WebSocket send → server stores and notifies the account → other device pulls →
  `decodePullResponse` (Rust) → `applyRemoteUpdate` (Rust) →
  `MemoryStore` merges Loro blob → note list updates
- Pushes remain in an account-scoped pending queue until the server's binary
  acknowledgement arrives. Disconnects reject the in-flight attempt and the
  reconnect loop retries it.
- Only a successfully applied pull advances `last_seen_global_seq`. A push
  acknowledgement never advances the pull cursor, preventing unseen remote
  updates from being skipped.
- Note storage, sync cursor, pending pushes, and device ID are scoped by
  account in `localStorage`.
- Deletes use the same durable pending queue as edits. A fresh client replays
  the append-only history in sequence, so the tombstone removes the note after
  its earlier updates and prevents resurrection on login or refresh.
- WASM runtime tests cover protocol encode/decode and remote update/delete.

### Native client (Android foundation implemented)

- `crates/core/src/sync/client.rs` owns a blocking `tungstenite` connection on
  a dedicated thread, reconnects with bounded backoff, and uses the same
  `protocol.rs` framing and pull decoder as web.
- `crates/core/src/storage/native.rs` combines SQLite-backed note operations
  with a durable, generation-checked sync outbox, device ID, and pull cursor.
- `crates/android-ffi/` exposes the store and sync session through UniFFI.
  `SyncDelegate` callbacks notify Kotlin about connection state and remote note
  changes; Kotlin dispatches those notifications to the main coroutine scope.
- `apps/android/` builds the Rust library for arm64, armv7, and x86_64 and
  packages the generated Kotlin bindings and native libraries into the APK.
- Periodic WorkManager refresh remains future work; the current client keeps a
  live socket and syncs while the application process is running.

## Open design items

- **Physical tombstone purge under E2E.** Logical deletion now syncs with a
  durable tombstone. The note's prior updates and tombstone must eventually
  vanish from the server's append-only log too. Direction: a "purge document"
  server call once all devices have acked. Design in Phase 2 before the
  protocol freezes.
- **Schema & format migrations.** Three things version independently:
  SQLite schema (numbered `.sql`-file migrations auto-discovered by
  `migration-build` — see [tech-stack.md](tech-stack.md)), Loro
  document format, and the sync wire format. Old app versions on other
  devices must fail gracefully, not corrupt. Plan in Phase 0.
