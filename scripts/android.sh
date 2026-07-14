#!/usr/bin/env bash
set -euo pipefail

# One-command Android development environment:
#   - starts the Rust API on the host at http://localhost:3000
#   - boots or reuses an Android emulator
#   - forwards the device's localhost:3000 to the host with adb reverse
#   - builds, installs, and launches the debug app
#
# Override the emulator with: AVD_NAME=Pixel_Tablet_API_35 ./scripts/android.sh

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SDK="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
ADB="$SDK/platform-tools/adb"
EMULATOR="$SDK/emulator/emulator"
PACKAGE="app.nononsense.notes"
SERVER_PID=""
EMULATOR_PID=""
SERIAL=""

require_command() {
  [[ -x "$1" ]] || { echo "Missing required executable: $1" >&2; exit 1; }
}

cleanup() {
  echo
  echo "Shutting down Android development server..."
  if [[ -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  if [[ "${STOP_EMULATOR_ON_EXIT:-0}" == "1" && -n "$EMULATOR_PID" && -n "$SERIAL" ]]; then
    "$ADB" -s "$SERIAL" emu kill >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

require_command "$ADB"
require_command "$EMULATOR"

if command -v lsof >/dev/null 2>&1 && lsof -nP -iTCP:3000 -sTCP:LISTEN >/dev/null 2>&1; then
  echo "==> Reusing the server already listening on http://localhost:3000"
else
  echo "==> Starting Rust server on http://localhost:3000"
  (
    cd "$ROOT"
    RUST_LOG=info cargo run -p no-nonsense-notes-server
  ) &
  SERVER_PID=$!
fi

SERIAL="$($ADB devices | awk '$1 ~ /^emulator-/ && $2 == "device" { print $1; exit }')"
if [[ -z "$SERIAL" ]]; then
  AVD_NAME="${AVD_NAME:-}"
  if [[ -z "$AVD_NAME" ]]; then
    AVD_NAME="$($EMULATOR -list-avds 2>/dev/null | grep -v '^INFO' | head -n 1 || true)"
  fi
  if [[ -z "$AVD_NAME" ]]; then
    echo "No Android Virtual Device found. Create one in Android Studio's Device Manager." >&2
    exit 1
  fi

  echo "==> Starting emulator: $AVD_NAME"
  "$EMULATOR" -avd "$AVD_NAME" -no-snapshot-save >"${TMPDIR:-/tmp}/no-nonsense-notes-emulator.log" 2>&1 &
  EMULATOR_PID=$!

  echo "==> Waiting for the emulator to appear"
  for _ in $(seq 1 120); do
    SERIAL="$($ADB devices | awk '$1 ~ /^emulator-/ { print $1; exit }')"
    [[ -n "$SERIAL" ]] && break
    sleep 1
  done
  [[ -n "$SERIAL" ]] || { echo "Emulator did not appear within 120 seconds." >&2; exit 1; }
fi

echo "==> Waiting for Android to finish booting ($SERIAL)"
$ADB -s "$SERIAL" wait-for-device
for _ in $(seq 1 180); do
  [[ "$($ADB -s "$SERIAL" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" == "1" ]] && break
  sleep 1
done
[[ "$($ADB -s "$SERIAL" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" == "1" ]] || {
  echo "Android did not finish booting within 180 seconds." >&2
  exit 1
}

echo "==> Forwarding device localhost:3000 to the host server"
$ADB -s "$SERIAL" reverse tcp:3000 tcp:3000

echo "==> Building and installing the debug APK"
(
  cd "$ROOT/apps/android"
  ./gradlew installDebug -PdebugApiUrl=http://127.0.0.1:3000
)

echo "==> Launching No Nonsense Notes"
if [[ -n "$SERVER_PID" ]] && ! kill -0 "$SERVER_PID" 2>/dev/null; then
  echo "The Rust server exited before the app could launch." >&2
  exit 1
fi
$ADB -s "$SERIAL" shell am force-stop "$PACKAGE"
$ADB -s "$SERIAL" shell am start -n "$PACKAGE/.MainActivity" >/dev/null

echo
echo "  Backend: http://localhost:3000"
echo "  Device:  $SERIAL"
echo "  App:     $PACKAGE"
echo
echo "Press Ctrl-C to stop the Rust server. The emulator stays open."
echo "Set STOP_EMULATOR_ON_EXIT=1 to close an emulator started by this script."

if [[ -n "$SERVER_PID" ]]; then
  wait "$SERVER_PID"
else
  while true; do sleep 60; done
fi
