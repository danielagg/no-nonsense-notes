# Domain Model & Ubiquitous Language

BDD + DDD. One shared vocabulary across code, docs, tests, and
conversation. Each bounded context owns its language; terms mean
the same thing everywhere they appear.

## Glossary

### Note

The central aggregate. A markdown document owned by an account,
identified by a unique `note_id` (UUIDv7). Stored as a Loro CRDT
doc blob inside SQLite. Metadata lives in dedicated relational
tables (not inside Loro).

| Field | Type | Description |
|---|---|---|
| `id` | UUIDv7 | Primary key, generated client-side |
| `folder_id` | UUIDv7 (nullable) | Parent folder; null = root |
| `title` | String | Derived from first `# Heading` or `Untitled` |
| `content_plaintext` | String | Extracted plaintext for FTS5 indexing |
| `content_loro_blob` | Blob | The full Loro document |
| `content_hash` | Bytes | SHA-256 of plaintext, for dedup-on-export |
| `created_at` | Timestamp (UTC) | Client-generated on first save |
| `updated_at` | Timestamp (UTC) | Updated on every edit |
| `is_deleted` | Boolean | Soft-delete flag (tombstone) |
| `deleted_at` | Timestamp (UTC, nullable) | When soft-deleted |
| `sort_order` | Integer | Manual sort position within folder |

**Rules:**
- A note belongs to exactly zero or one folder (flat hierarchy -- no
  nested folders in v1).
- `title` is never stored directly; it is derived from the first
  `# Heading` in the markdown content, or falls back to `"Untitled"`.
- `content_plaintext` is regenerated on every save for FTS5 indexing.
- `content_loro_blob` is the serialized Loro doc -- this is what syncs.
- `is_deleted` notes are hidden from the UI but kept for tombstone
  protocol until all devices ack.

### Tag

A label applied to notes. Many-to-many.

| Field | Type | Description |
|---|---|---|
| `id` | UUIDv7 | Primary key |
| `name` | String (unique) | Display name, case-insensitive unique per account |
| `color` | String (nullable) | Optional hex color |

Join table: `note_tags (note_id, tag_id)`.

### Folder

An organization bucket for notes. Flat list -- no nesting in v1.

| Field | Type | Description |
|---|---|---|
| `id` | UUIDv7 | Primary key |
| `name` | String | Display name |
| `sort_order` | Integer | Manual sort position |
| `created_at` | Timestamp (UTC) | |

**Rules:**
- Folder names are unique per account.
- Deleting a folder sets contained notes' `folder_id` to null (notes
  go to root, not deleted).
- No trash/recycle bin in v1.

### Account

A user identity with email + password. Owns all notes, folders, tags,
and devices.

| Field | Type | Description |
|---|---|---|
| `id` | UUIDv7 | Primary key |
| `email` | String (unique) | Login identifier |
| `auth_key_hash` | Bytes | Server-stored hash of the derived auth key |
| `password_salt` | Bytes | Per-account salt for Argon2id |
| `argon2_params` | String | Serialized Argon2id params (memory, iterations, parallelism) |
| `created_at` | Timestamp (UTC) | |

**Rules:**
- Account is the root aggregate for auth -- devices and sync state
  hang off it.
- `auth_key_hash` is what the server stores; the actual auth key is
  derived client-side and never persisted.
- Email is the only identifier. No username, no phone number, no OAuth
  in v1.

### Device

A specific installation of the app belonging to an account.

| Field | Type | Description |
|---|---|---|
| `id` | UUIDv7 | Primary key |
| `account_id` | UUIDv7 | FK to account |
| `name` | String | Human-readable (e.g. "Pixel 7", "MacBook Pro") |
| `public_key` | Bytes | ed25519 public key (device identity) |
| `encrypted_private_key` | Bytes | ed25519 private key, encrypted with master key |
| `paired_at` | Timestamp (UTC) | When this device joined the account |
| `last_synced_at` | Timestamp (UTC, nullable) | Last successful sync |

**Rules:**
- Each device has its own ed25519 identity keypair.
- The private key is encrypted with the master key at rest.
- Device identity signs sync payloads so the server can verify the
  source device (prevents replay across devices).

### SyncState

Per-device tracking of sync progress.

| Field | Type | Description |
|---|---|---|
| `device_id` | UUIDv7 | FK to device |
| `last_seen_global_seq` | Integer | Highest global sequence number pulled from server |
| `last_pushed_seq` | Integer | Highest sequence number pushed to server |
| `dirty` | Boolean | True if local changes exist that haven't been pushed |

**Rules:**
- `last_seen_global_seq` is the cursor for pull: "give me everything
  after this number."
- On reconnect, the client pushes all dirty changes, then pulls from
  `last_seen_global_seq + 1`.

### Update

A single entry in the server's append-only change log.

| Field | Type | Description |
|---|---|---|
| `global_seq` | Integer (auto-increment) | Monotonic, server-assigned |
| `doc_id` | UUIDv7 | Which note this update belongs to |
| `device_id` | UUIDv7 | Which device produced this update |
| `encrypted_blob` | Blob | XChaCha20-Poly1305 encrypted Loro update |
| `created_at` | Timestamp (UTC) | Server-assigned |

**Rules:**
- `global_seq` is the only cursor the client tracks.
- Server never inspects `encrypted_blob`.
- Updates are never modified or deleted (append-only until tombstone
  purge).

### Settings

Per-device configuration that is not synced across devices.

| Key | Type | Default | Description |
|---|---|---|---|
| `theme` | Enum | `system` | `light`, `dark`, `system` |
| `font_size` | Integer | `16` | Editor font size in px |
| `default_folder_id` | UUIDv7 (nullable) | `null` | New notes created here by default |
| `sort_notes_by` | Enum | `updated_at` | `title`, `created_at`, `updated_at` |

Settings are stored in a simple key-value SQLite table scoped to
`device_id`. They are not synced (intentional -- different devices
may have different preferences).

---

## Bounded Contexts

### Storage Context

Owns note CRUD, folder/tag management, search, import/export.
Does not know about sync or encryption.

**Commands:**
- `CreateNote(folder_id?) -> Note`
- `EditNote(id, new_content) -> Note`
- `DeleteNote(id) -> void` (soft-delete, sets `is_deleted = true`)
- `MoveNote(id, folder_id) -> Note`
- `SearchNotes(query) -> [Note]`
- `ExportNotes(ids) -> [MarkdownFile]`
- `ImportNotes(files) -> [Note]`

**Queries:**
- `ListNotes(folder_id?, tag_id?, sort_order) -> [Note]`
- `ListFolders() -> [Folder]`
- `ListTags() -> [Tag]`

### Sync Context

Owns the change-log protocol. Encrypts/decrypts Loro updates, manages
the WebSocket connection, offline queue, and reconnect logic. Depends
on Storage (to read/write Loro blobs) and Crypto (to encrypt/decrypt).

**Events:**
- `SyncConnected` -- WebSocket opened
- `SyncDisconnected(reason)` -- WebSocket closed or errored
- `RemoteUpdatesReceived(updates)` -- new blobs pulled from server
- `LocalChangesPushed(count)` -- our changes acked by server
- `SyncError(error)` -- unrecoverable (e.g. auth failure)

### Crypto Context

Owns key derivation, encryption/decryption, device identity, and the
QR pairing handshake. No knowledge of notes or sync -- operates on
opaque byte blobs.

**Operations:**
- `DeriveKeys(password, salt) -> (auth_key, master_key)`
- `EncryptNote(plaintext, doc_key) -> ciphertext`
- `DecryptNote(ciphertext, doc_key) -> plaintext`
- `WrapDocKey(doc_key, master_key) -> wrapped_key`
- `UnwrapDocKey(wrapped_key, master_key) -> doc_key`
- `GenerateDeviceKeypair() -> (public_key, encrypted_private_key)`
- `PairDevice(qr_code_data) -> DeviceIdentity`

### Editor Context

Owns markdown parsing, syntax highlighting, rendering, and the UI
editing experience. Depends on Storage (to save content) but not on
Sync or Crypto.

**Operations:**
- `ParseMarkdown(content) -> SyntaxSpans`
- `RestyleRange(content, changed_range) -> SyntaxSpans` (incremental)
- `RenderTable(content) -> TableLayout`
- `ToggleChecklist(line_number) -> new_content`

### Auth Context

Owns account lifecycle: signup, signin, session management.
Depends on Crypto (for key derivation).

**Commands:**
- `SignUp(email, password) -> Account`
- `SignIn(email, password) -> Session`
- `SignOut() -> void`
- `RegisterDevice(name) -> Device`
- `RemoveDevice(device_id) -> void`

---

## DDD/BDD Approach

### Aggregate Design

```
Account (root)
  ├── Device (child)
  │     └── SyncState (child)
  └── Settings (child, per-device)

Note (root)
  ├── Tag (via note_tags join)
  └── Folder (reference)
```

- **Account** and **Note** are independent aggregates.
- Notes reference a Folder by ID (not a child entity).
- Tags are independent entities connected via join table.
- SyncState is a value object under Device.

### Behavior-Driven Scenarios

Features are specified as Gherkin scenarios living alongside the
implementation. Example convention:

```
# features/note-crud.feature
Feature: Note lifecycle
  As a user
  I want to create, edit, and delete notes
  So that I can capture and organize my thoughts

  Scenario: Create a note with a title
    Given I have an empty folder "Work"
    When I create a note with content "# Meeting Notes"
    Then the note appears in the "Work" folder
    And its title is "Meeting Notes"
    And its content is stored as a Loro document

  Scenario: Edit an existing note
    Given a note with content "# Old Title"
    When I change the content to "# New Title\n\nUpdated body."
    Then the note's title updates to "New Title"
    And the Loro document contains the update
    And the note's updated_at timestamp changes

  Scenario: Soft-delete a note
    Given a note "Temp Note"
    When I delete "Temp Note"
    Then the note is hidden from all folder/tag views
    And the note has is_deleted = true
    And the Loro document is preserved for tombstone sync
```

### Test Mapping

BDD scenarios map directly to the test categories in
[testing.md](testing.md):

| Scenario type | Test category |
|---|---|
| Note CRUD scenarios | Unit tests (Storage Context) |
| Concurrent edit + merge | Property-based tests (CRDT) |
| Sync round-trip scenarios | Sync round-trip tests |
| Encryption + key derivation | Crypto tests |
| Editor rendering | UI smoke tests (platform-specific) |

### Design Heuristics

- **Cross-context references are by ID only.** The Sync context
  references a `doc_id` but never loads a Note entity.
- **Each context may have its own persistence view.** Sync writes to
  the `update_log` table on the server; Storage writes to the
  `notes` table on the client. They do not share tables.
- **Value objects are immutable.** `DocumentKey`, `GlobalSeqNum`,
  `SyntaxSpans` are created and never mutated.
- **Aggregate roots enforce invariants.** A Note cannot exist without
  a `note_id` or `content_loro_blob`. An Account cannot exist without
  an `auth_key_hash` and `password_salt`.
