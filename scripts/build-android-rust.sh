#!/usr/bin/env bash
set -euo pipefail

# Gradle's internal Rust/UniFFI build step. For the full local development
# workflow, run ./scripts/android.sh instead.

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ANDROID_DIR="$ROOT/apps/android/app/src/main"
NDK="${ANDROID_NDK_HOME:-${ANDROID_HOME:-$HOME/Library/Android/sdk}/ndk/26.1.10909125}"
HOST_TAG="darwin-x86_64"
[[ "$(uname -s)" == "Linux" ]] && HOST_TAG="linux-x86_64"
TOOLCHAIN="$NDK/toolchains/llvm/prebuilt/$HOST_TAG/bin"
API=26

rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

build_target() {
  local rust_target="$1" clang_prefix="$2" abi="$3" linker_env="$4"
  env \
    "CARGO_TARGET_${linker_env}_LINKER=$TOOLCHAIN/${clang_prefix}${API}-clang" \
    "CC_${rust_target//-/_}=$TOOLCHAIN/${clang_prefix}${API}-clang" \
    "AR_${rust_target//-/_}=$TOOLCHAIN/llvm-ar" \
    cargo build -p no-nonsense-notes-android --release --target "$rust_target"
  mkdir -p "$ANDROID_DIR/jniLibs/$abi"
  cp "$ROOT/target/$rust_target/release/libno_nonsense_notes_android.so" "$ANDROID_DIR/jniLibs/$abi/"
}

cd "$ROOT"
build_target aarch64-linux-android aarch64-linux-android arm64-v8a AARCH64_LINUX_ANDROID
build_target armv7-linux-androideabi armv7a-linux-androideabi armeabi-v7a ARMV7_LINUX_ANDROIDEABI
build_target x86_64-linux-android x86_64-linux-android x86_64 X86_64_LINUX_ANDROID

cargo build -p no-nonsense-notes-android
cargo run -p no-nonsense-notes-uniffi-bindgen -- generate \
  "$ROOT/target/debug/libno_nonsense_notes_android.$([[ "$(uname -s)" == "Darwin" ]] && echo dylib || echo so)" \
  --language kotlin --out-dir "$ANDROID_DIR/java" --no-format

