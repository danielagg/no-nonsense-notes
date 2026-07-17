# Android app

The native Android client. It uses Kotlin and Jetpack Compose for the UI and
calls the shared Rust core through UniFFI.

Implemented features include authentication, account-scoped SQLite storage,
markdown notes, checklists, search, autosave, offline changes, and reconnecting
WebSocket sync.

## Run locally

Requirements: JDK 17, Android SDK 34, NDK 26.1, an Android emulator, and the
Rust Android targets.

From the repository root:

```sh
./scripts/android.sh
```

The script starts or reuses the Rust server, boots or reuses an emulator,
configures `adb reverse`, installs the debug APK, and launches the app. Set
`AVD_NAME` to select an emulator.

## Build

From `apps/android`:

```sh
./gradlew assembleDebug
./gradlew assembleRelease
```

`preBuild` runs [`scripts/build-android-rust.sh`](../../scripts/build-android-rust.sh).
That script builds the Rust libraries for arm64, armv7, and x86_64 and
regenerates the Kotlin UniFFI bindings.

The generated file under `app/nononsense/notes/core/` should not be edited by
hand. Change the API in [`crates/android-ffi`](../../crates/android-ffi/README.md)
and rebuild instead.

## Backend URL

- Debug default: `http://10.0.2.2:3000`
- Release default: `https://no-nonsense-notes.onrender.com`

Override either value at build time:

```sh
./gradlew assembleDebug -PdebugApiUrl=http://127.0.0.1:3000
./gradlew assembleRelease -PreleaseApiUrl=https://example.com
```

Release signing is enabled when the four `NNN_ANDROID_KEYSTORE*` environment
variables referenced in `app/build.gradle.kts` are present. Without them,
Gradle produces an unsigned release APK.
