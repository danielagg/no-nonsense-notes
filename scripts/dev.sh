#!/usr/bin/env bash
set -euo pipefail

# Start both the Rust server and the web dev server from the project root.
# Builds WASM first (required — wasm-pkg/ is gitignored).
# Usage: ./scripts/dev.sh

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "==> Building WASM ..."
wasm-pack build --target web --out-dir pkg "$ROOT_DIR/crates/wasm"
cp "$ROOT_DIR/crates/wasm/pkg/no_nonsense_notes_wasm.js" \
   "$ROOT_DIR/crates/wasm/pkg/no_nonsense_notes_wasm.d.ts" \
   "$ROOT_DIR/crates/wasm/pkg/no_nonsense_notes_wasm_bg.wasm" \
   "$ROOT_DIR/crates/wasm/pkg/no_nonsense_notes_wasm_bg.wasm.d.ts" \
   "$ROOT_DIR/apps/web/src/lib/wasm-pkg/"
echo "    WASM built and copied."

cleanup() {
  echo ""
  echo "Shutting down..."
  kill "$SERVER_PID" "$WEB_PID" 2>/dev/null || true
  wait "$SERVER_PID" "$WEB_PID" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

echo "==> Starting Rust server on :3000 ..."
RUST_LOG=info cargo run -p no-nonsense-notes-server --manifest-path "$ROOT_DIR/Cargo.toml" &
SERVER_PID=$!

echo "==> Starting web dev server ..."
cd "$ROOT_DIR/apps/web" && npm run dev &
WEB_PID=$!

echo ""
echo "  Server:  http://localhost:3000"
echo "  Swagger: http://localhost:3000/swagger-ui"
echo "  Web app: http://localhost:5173"
echo ""

wait
