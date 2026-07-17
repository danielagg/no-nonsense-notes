# Android FFI crate

`no-nonsense-notes-android` is the UniFFI boundary between the shared Rust core
and the Kotlin Android app.

It exports:

- Android-friendly note records and enums
- `NotesStore` for local note operations
- `SyncSession` and `SyncDelegate` for background synchronization
- Errors converted into a stable foreign-language representation

This crate is built as both an `rlib` and a `cdylib`. The Android Gradle build
compiles it for each supported ABI and regenerates the Kotlin bindings.

## Commands

```sh
cargo check -p no-nonsense-notes-android
./scripts/build-android-rust.sh
```

Keep platform-neutral behavior in [`core`](../core/README.md). This crate should
remain a small translation layer for types, ownership, and callbacks.
