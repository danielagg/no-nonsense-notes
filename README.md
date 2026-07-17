# No Nonsense Notes

Fast, local-first markdown notes and checklists with CRDT sync. The business
logic lives in Rust and is shared by the Android and web clients.

The project is under active development. Android, web, the shared core, and the
sync server are implemented; macOS and iOS remain planned work.

## Quick start

Configure the server and web URLs using [`.env.example`](.env.example) and
[`apps/web/.env.example`](apps/web/.env.example), then run:

```sh
./scripts/web.sh
```

This builds the WASM bridge and starts the server on port 3000 and the web app
on port 5173. For Android development, use:

```sh
./scripts/android.sh
```

See the app READMEs for prerequisites and platform-specific commands.

## Workspace

### Apps

| App | Purpose |
|---|---|
| [Android](apps/android/README.md) | Native Jetpack Compose client using the Rust core through UniFFI |
| [Web](apps/web/README.md) | React client using the Rust core through WASM |
| [macOS](apps/macos/README.md) | Planned SwiftUI client |
| [iOS](apps/ios/README.md) | Planned SwiftUI client |

### Rust crates

| Crate | Purpose |
|---|---|
| [`core`](crates/core/README.md) | Notes, storage, CRDTs, search, and sync logic |
| [`server`](crates/server/README.md) | Authentication and WebSocket sync service |
| [`android-ffi`](crates/android-ffi/README.md) | UniFFI API consumed by the Android app |
| [`wasm`](crates/wasm/README.md) | `wasm-bindgen` API consumed by the web app |
| [`migration-build`](crates/migration-build/README.md) | Shared SQLite migration generator and runner |
| [`uniffi-bindgen`](crates/uniffi-bindgen/README.md) | Workspace-pinned UniFFI binding generator |

## Architecture

```text
Android / Kotlin ── UniFFI ──┐
                              ├── Rust core ── WebSocket sync ── Rust server
Web / React ── wasm-bindgen ──┘
```

Native storage uses SQLite. The web client uses the core's in-memory store with
`localStorage` persistence. Loro documents provide conflict-free note merging.

## Common checks

```sh
cargo check --workspace
cargo test --workspace

cd apps/web
npm run lint
npm run test
npm run build

cd ../android
./gradlew assembleDebug
```

## Design docs

- [Project context](CONTEXT.md)
- [Tech stack](docs/tech-stack.md)
- [Sync](docs/sync.md)
- [Security](docs/security.md)
- [Editor](docs/editor.md)
- [Testing](docs/testing.md)
- [Roadmap](docs/roadmap.md)

## License

MIT
