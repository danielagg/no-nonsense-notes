# Android app

Native Kotlin/Jetpack Compose client backed by the shared Rust core through UniFFI.

## Run locally

From the repository root:

```sh
./scripts/android.sh
```

The script starts the Rust server on `http://localhost:3000`, boots or reuses an
emulator, forwards the device's port 3000 to the host with `adb reverse`, builds
and installs the debug APK, and launches the app. Set `AVD_NAME` to choose a
specific virtual device.

## Build an APK

Requirements: JDK 17+, Android SDK 34, NDK 26.1, and Rust Android targets. From this directory:

```sh
./gradlew assembleDebug
```

The debug APK is written to `app/build/outputs/apk/debug/app-debug.apk`. It is
signed with Android's development key and defaults to the emulator host alias
`http://10.0.2.2:3000`. Override it at build time with:

```sh
./gradlew assembleDebug -PdebugApiUrl=http://127.0.0.1:3000
```

For a production release:

```sh
./gradlew assembleRelease
```

Release builds compile `https://no-nonsense-notes.onrender.com` into
`BuildConfig.API_URL` and disable cleartext HTTP. Without signing environment
variables, Gradle produces `app/build/outputs/apk/release/app-release-unsigned.apk`.
For a signed APK, set:

```sh
export NNN_ANDROID_KEYSTORE=/absolute/path/to/release.jks
export NNN_ANDROID_KEYSTORE_PASSWORD=...
export NNN_ANDROID_KEY_ALIAS=...
export NNN_ANDROID_KEY_PASSWORD=...
./gradlew assembleRelease
```

The signed output is `app/build/outputs/apk/release/app-release.apk`. The backend
URL is a compile-time build setting, not a deployment-time injection. Override
the production default when needed with
`-PreleaseApiUrl=https://another.example.com`.

`preBuild` runs `scripts/build-android-rust.sh`, which builds Rust `.so` files for arm64, armv7, and x86_64 and regenerates the Kotlin UniFFI bindings.

The current slice includes auth, account-scoped SQLite storage, note/list CRUD, FTS search, autosave, live markdown styling, a persistent sync outbox, reconnecting WebSocket sync, and offline/sync status UI.
