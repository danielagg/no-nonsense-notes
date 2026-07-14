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
- Two connected sessions for one account receive update notifications and
  pull committed edits without manual refresh
- Push acknowledgements do not advance the pull cursor past unseen updates

## Crypto verification tests

- Assert ciphertext != plaintext (every encryption produces different
  bytes from input)
- Decryption round-trip: decrypt(encrypt(plaintext)) == plaintext
- Key derivation: stable for same input, different for different salt
- Zeroization test: key material memory reads as zeroed after drop
  (using mlock/mprotect if available, else best-effort)

## Web tests (`vitest`)

The web app has a vitest test suite in `apps/web/src/`. Tests mock the
WASM module and verify the TypeScript wrapper logic:

- API routing: `updateMarkdownNote` calls `wasmUpdateNote`, not
  `wasmUpdateList` (and vice versa for list notes)
- `wasmToNote` mapping: markdown notes get `items=undefined`, list
  notes get `items` array from `contentPlaintext.split('\n')`
- Auth and search call the right WASM methods; delete soft-deletes locally and
  queues a sync tombstone

Run: `npm run test` (or `npm run test:watch` for watch mode) in
`apps/web/`.

## WASM runtime tests (`wasm-pack test`)

The WASM crate has `wasm-bindgen-test` tests that run in a headless
browser. These catch platform-specific issues that `cargo check` and
`cargo test` miss:

- `chrono::Utc::now()` panics on `wasm32-unknown-unknown` without the
  `wasmbind` feature -- the create-note test catches this
- `getrandom` needs `wasm_js` backend -- UUIDv7 generation exercises this
- Note CRUD round-trips through the `WasmStore` + `MemoryStore` stack
- User-owned titles survive content/list edits and remote CRDT replay; list
  items never become note titles
- Tombstone frames decode as deletes, and remote deletion is idempotent for
  both existing and already-absent notes

Run: `wasm-pack test --headless --chrome --cargo-arg=--no-default-features
crates/wasm` (requires Chrome installed).

## CI layout (GitHub Actions)

```
Linux runner:
  - cargo check --workspace --all-features
  - cargo check --no-default-features --target wasm32-unknown-unknown -p no-nonsense-notes-wasm
  - cargo test --workspace
  - wasm-pack build --target web --out-dir pkg crates/wasm
  - wasm-pack test --headless --chrome --cargo-arg=--no-default-features crates/wasm
  - npm run lint (apps/web)
  - npm run test (apps/web, vitest)
  - npm run build (apps/web, tsc + vite)
  - ./gradlew assembleDebug (apps/android; builds UniFFI + three Rust ABIs)

Deploy (only if tests pass):
  - Trigger Vercel deploy via webhook
```

No separate QA environment for v1. Dogfooding on the primary device
is the main integration test (Phase 1 exit criterion).

## Deployment

- **Render.com** (free tier) — auto-deploys on push to `main` after
  CI passes
- Docker-based build from repo root `Dockerfile`
- Free tier: no persistent disk, SQLite rebuilt on cold start
- CI triggers Render deploy via webhook after tests pass
