# Security & Encryption

Goal: end-to-end encrypted by design. The server stores and relays
ciphertext only -- it can never read note contents or metadata.
See [sync.md](sync.md) for how encrypted updates move,
[tech-stack.md](tech-stack.md) for the overall stack.

## Cryptography libraries (RustCrypto)

- chacha20poly1305 (XChaCha20-Poly1305, content encryption)
- x25519-dalek (key exchange during device pairing)
- ed25519-dalek (device identity / signatures)
- argon2 (password key derivation)
- hkdf (key splitting)
- sha2
- zeroize (memory zeroization for sensitive material -- defense-in-depth
  against memory dumps, even though the primary threat model is
  server-side)
- getrandom

## End-to-end encryption

- All note and metadata content encrypted client-side with
  XChaCha20-Poly1305 before it leaves the device
- Per-document keys, wrapped by the master key
- Server stores only ciphertext -- verified by test, not assumed
  (Phase 2b exit criterion)
- Sensitive key material zeroized in memory after use

## Authentication & keys (Standard Notes / Bitwarden pattern)

One password does both jobs; the server never sees the encryption key:

1. `password + salt -> Argon2id -> root key`
2. `root key -> HKDF -> auth key + master key`
3. **auth key** is sent to the server for login (server stores only a
   hash of it)
4. **master key** wraps per-document keys; it never leaves the device
   except during QR pairing, where it is exchanged encrypted via x25519
   between two devices

### Account creation

Email + password. No email verification, no magic link, no password
reset in v1. The password is the root of all key material -- losing it
loses the notes (stated clearly in the UI at signup). Stated v1
limitations:

- No password reset flow
- No email verification
- No account recovery

## Device identity & pairing

- ed25519 keypair per device (identity)
- New device gets the master key via QR-code pairing with an existing
  device (x25519 key exchange) -- the server never sees it
- Short dedicated design pass before implementation (Phase 2b)

## Key recovery

- **v1: accepted data loss.** Losing all devices (or the password)
  means losing the notes. Stated clearly in the UI at signup.
- Recovery phrase (12/24 words) planned post-v1
  (see [roadmap.md](roadmap.md))

## Transport

- TLS terminated by Caddy (automatic Let's Encrypt) in front of the
  Rust backend; authenticated WebSockets inside

## Open questions / watch-outs

- **Local at-rest encryption.** Content is encrypted before it leaves
  the device, but SQLite on disk is plaintext -- which is what lets
  FTS5 index it directly. Decide whether a lost/stolen device is in
  scope for the threat model. If so, design it deliberately
  (OS keychain-backed at-rest encryption), not as a retrofit.
- **Multi-device key management.** The QR pairing flow needs its
  design pass before Phase 2b code: exact handshake, what the server
  mediates (only opaque messages), how pairing is confirmed on both
  screens.
- **Compression of sync payloads** was deliberately cut from v1:
  marginal gain, and compress-then-encrypt leaks size patterns.
  Revisit with care post-v1.
- **Password change / key rotation.** The master key wraps
  per-document keys, so a password change should only require deriving
  a new root key and re-wrapping -- but no flow is designed yet.
  Affects the storage schema (wrapped keys must be re-writable and
  versioned), so note the shape in Phase 0 even though implementation
  is Phase 2b.
- **Session / WebSocket auth lifecycle.** Decided: opaque bearer tokens
  (not JWT). Server generates a random token on successful signin, stores
  it in `auth_tokens` table. Client sends it as `Authorization: Bearer
  <token>` on HTTP requests and as the first message after WebSocket
  upgrade. Token has no expiry in v1 (simple; revocation via device
  removal). Refresh and revocation design deferred to Phase 2b alongside
  device pairing.
- **Account features beyond email+password.** Rate limiting, password
  reset, email verification, account deletion -- none designed yet.
  Keep scope minimal for v1.
