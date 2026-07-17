# Web app

The React and TypeScript client. It uses the shared Rust core through
`wasm-bindgen`, keeps local notes in the WASM `MemoryStore`, persists them to
`localStorage`, and connects to the Rust sync server.

## Run locally

Set `VITE_API_URL` using [`.env.example`](.env.example), then run this from the
repository root:

```sh
./scripts/web.sh
```

The script builds [`crates/wasm`](../../crates/wasm/README.md), copies the
generated package into `src/lib/wasm-pkg/`, and starts both the server and Vite.

## Commands

From `apps/web`:

```sh
npm install
npm run dev
npm run lint
npm run test
npm run build
npm run preview
```

Running Vite directly assumes `src/lib/wasm-pkg/` has already been generated.
Use `scripts/web.sh` after Rust or WASM API changes.

## Structure

- `src/components/` contains the auth, note-list, and editor UI.
- `src/lib/wasm.ts` wraps the generated WASM API.
- `src/lib/sync-manager.ts` owns the browser WebSocket connection.
- `src/lib/wasm-pkg/` is generated and should not be edited manually.
