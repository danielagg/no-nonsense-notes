# UniFFI bindgen crate

`no-nonsense-notes-uniffi-bindgen` is a thin workspace binary around the
UniFFI CLI. Keeping it in the workspace pins binding generation to the same
UniFFI version used by [`android-ffi`](../android-ffi/README.md).

The Android Rust build script invokes this binary to regenerate Kotlin bindings.
It is infrastructure, not application logic.

## Build

```sh
cargo build -p no-nonsense-notes-uniffi-bindgen
```
