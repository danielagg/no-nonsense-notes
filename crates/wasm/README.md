# WASM crate

`no-nonsense-notes-wasm` is the browser-facing `wasm-bindgen` API used by the
React app.

It wraps the core `MemoryStore`, persists account-scoped state in
`localStorage`, converts Rust notes into JavaScript objects, and exposes the
shared sync frame encoders and decoders.

## Build and test

From the repository root:

```sh
wasm-pack build --target web --out-dir pkg crates/wasm
wasm-pack test --headless --chrome crates/wasm --no-default-features
```

For a fast host-side compile check:

```sh
cargo check -p no-nonsense-notes-wasm
```

The build output under `pkg/` and the copy under
`apps/web/src/lib/wasm-pkg/` are generated. Use [`scripts/web.sh`](../../scripts/web.sh)
to rebuild and copy the package before running the web client.
