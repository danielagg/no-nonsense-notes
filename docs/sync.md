# CRDT & Sync

How notes stay consistent across devices without the server ever
reading them. Encryption details in [security.md](security.md).

## CRDT: Loro

- **Loro** (Rust-native CRDT), one document per note
- Metadata (folders, tags, note metadata, settings) stored in dedicated
  SQLite tables, not inside a Loro doc -- avoids decryption overhead for
  folder/tag listing and eliminates a single-writer contention point
  across devices
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
- Offline queueing, fast reconnect
- **Android:** sync-on-open (+ optional periodic WorkManager refresh);
  live WebSocket while the app is open. No FCM -- keeps the app
  Google-free and F-Droid-friendly. Push-based background sync
  (UnifiedPush / FCM) is post-v1.
- **macOS:** persistent WebSocket while running; sync is effectively
  real-time

## Open design items

- **Tombstones / deletion under E2E.** A deleted note must eventually
  vanish from the server's append-only log too. Direction: tombstone
  note in SQLite metadata + a "purge document" server call once all
  devices have acked. Design in Phase 2 before the protocol freezes.
- **Schema & format migrations.** Three things version independently:
  SQLite schema (numbered `.sql`-file migrations auto-discovered by
  `migration-build` — see [tech-stack.md](tech-stack.md)), Loro
  document format, and the sync wire format. Old app versions on other
  devices must fail gracefully, not corrupt. Plan in Phase 0.
