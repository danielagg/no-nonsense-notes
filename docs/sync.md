# CRDT & Sync

How notes stay consistent across devices without the server ever
reading them. Encryption details in [security.md](security.md).

## CRDT: Loro

- **Loro** (Rust-native CRDT), one document per note, plus one
  "workspace" document for all metadata (folders, tags, note metadata,
  settings) -- metadata syncs over the same path as notes, no second
  mechanism
- Chosen over Automerge: faster, and shallow snapshots solve the
  unbounded-history-growth problem for long-lived, heavily edited notes
- **Phase 0 gate:** benchmark a 10k-edit markdown document (load time,
  memory, update size) before building on top of it
- Documents stored as blobs in SQLite; FTS5 indexes extracted plaintext

## Sync protocol: encrypted change-log

Own thin protocol. The stock Loro sync protocol require both
peers to read document state (heads, bloom filters), which is
incompatible with a blind relay -- hence this design:

- Server keeps an **append-only log of encrypted Loro update blobs**
  per document, with a per-document sequence number
- Client pull: "give me everything after cursor N" -> decrypt ->
  `import` into the local Loro doc
- Client push: export local updates -> encrypt -> append to the
  server log
- The server never participates in CRDT logic and never sees plaintext
- Wire format: raw encrypted bytes (no JSON), with a **version field in
  every payload**

## Transport & behavior

- Authenticated **WebSockets** (instant push to connected devices),
  TLS via Caddy
- Offline queueing, fast reconnect
- **Android:** sync-on-open (+ optional periodic WorkManager refresh);
  live WebSocket while the app is open. No FCM -- keeps the app
  Google-free and F-Droid-friendly. Push-based background sync
  (UnifiedPush / FCM) is post-v1.
- **macOS:** persistent WebSocket while running; sync is effectively
  real-time

## Open design items

- **Tombstones / deletion under E2E.** A deleted note must eventually
  vanish from the server's append-only log too, not just be marked
  deleted in the CRDT. Direction: tombstone in the workspace doc + a
  server "purge document" call once all devices have acked. Design in
  Phase 2 before the protocol freezes.
- **Workspace-doc contention.** One Loro doc holds all metadata, so
  every rename, tag change, and note creation from every device writes
  to the same document -- it will be the hottest document in the
  system and its update log grows fastest. Fine at personal-notes
  scale, but keep an eye on it in the Phase 0 benchmark and prefer
  shallow snapshots if the log gets long.
- **Schema & format migrations.** Three things version independently:
  SQLite schema (numbered migrations from day one), Loro document
  format, and the sync wire format. Old app versions on other devices
  must fail gracefully, not corrupt. Plan in Phase 0.
