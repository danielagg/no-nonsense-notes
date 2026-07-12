# Testing & QA

Testing strategy for a solo-dev project: maximise confidence with
minimum maintenance burden. Heavy investment in the Rust core where
85% of the logic lives; lighter smoke tests on the UI layers.

## Rust core tests (`cargo test`)

### Unit tests

- Note CRUD: create, edit, delete, search, export
- Loro doc operations: import/export update blobs, shallow snapshots
- SQLite schema migrations: apply each migration, verify expected
  schema version, test idempotency (run twice, same result)
- Both `core` and `server` crates share the same migration convention
  (individual `.sql` files, auto-discovered by `migration-build` crate)
- Markdown parsing: syntax spans match expected ranges for all v1
  elements (headings, bold/italic, lists, checklists, blockquotes,
  tables, code blocks, links, horizontal rules)

### Property-based tests

- CRDT merge: concurrent edits on two Loro docs converge to identical
  final state (any sequence of operations)
- Encryption round-trip: any plaintext decrypts to its original form
  after encrypt → decrypt with the same key
- Key derivation: same password + salt always produces same root key;
  different salts produce different keys

## Benchmark gates

Fail the build if thresholds are not met:

| Benchmark | Threshold | When |
|---|---|---|
| 10k-edit Loro doc load time | < 500ms | Phase 0 (gate) |
| 10k-edit Loro doc memory | < 50 MB | Phase 0 (gate) |
| Editor typing latency (10k-line doc) | < 16ms per keystroke | Phase 1 |
| Sync throughput | > 1 MB/s encrypted blobs | Phase 2a |

## Sync round-trip tests

Runs in CI without real devices:

- Two in-process client cores + an in-process test server (same Axum
  binary)
- Simulate: create note on client A -> sync -> verify note exists on
  client B
- Concurrent offline edits on both clients -> reconnect -> verify
  Loro convergence
- Kill-the-server-mid-sync -> client retries -> recovery
- Fresh client pulls full history -> correct state

## Crypto verification tests

- Assert ciphertext != plaintext (every encryption produces different
  bytes from input)
- Decryption round-trip: decrypt(encrypt(plaintext)) == plaintext
- Key derivation: stable for same input, different for different salt
- Zeroization test: key material memory reads as zeroed after drop
  (using mlock/mprotect if available, else best-effort)

## CI layout (GitHub Actions)

```
Linux runner:
  - cargo test (core unit + property + crypto)
  - cargo bench (benchmark gates)
  - Sync round-trip tests (in-process)
  - Android build (Gradle + NDK, smoke only)

macOS runner:
  - Swift build (Phase 3+)
  - macOS app smoke test
```

No separate QA environment for v1. Dogfooding on the primary device
is the main integration test (Phase 1 exit criterion).
