# No Nonsense Notes

Local-first, E2E-encrypted markdown and list notes with CRDT sync. The app
must work offline-first, sync when connected, and keep all content encrypted
end-to-end.

## Language

### Note

The central aggregate. A document owned by an account, identified by a
UUIDv7 `note_id`. Two variants exist: **Markdown** (prose) and **List**
(checklist/todo). Each variant stores its content in a different Loro CRDT
container — text for Markdown, list for List. Most metadata lives in relational
tables. The user-owned title is the exception: it lives in the Loro document's
`metadata.title` field for CRDT sync and is mirrored in SQLite for listing/FTS.

_Avoid_: Document, entry, page

### NoteType

The variant of a note: `Markdown` or `List`. Determines which Loro
container is used and which operations are available. Markdown notes use
`EditNote` (whole-content replacement). List notes use `ListAddItem` and
`ListRemoveItem` (item-level CRUD). A note's type is fixed at creation and
cannot change.

_Avoid_: Kind, variant, format

### Folder

An organization bucket for notes. Flat list — no nesting in v1.

_Avoid_: Category, collection, group

### Tag

A label applied to notes. Many-to-many. Tags are unique per account (not
globally — the database constraint must enforce per-account uniqueness
once multi-account support ships).

_Avoid_: Label, keyword, hashtag

### Account

A user identity with email + password. Owns all notes, folders, tags, and
devices. The root aggregate for auth — devices and sync state hang off it.

_Aavoid_: User, identity, profile

### Device

A specific installation of the app belonging to an account. Each device
has its own ed25519 identity keypair. The private key is encrypted with
the master key at rest. Device identity signs sync payloads so the server
can verify the source device.

_Avoid_: Installation, client, endpoint

### SyncState

Per-device tracking of sync progress. Tracks `last_seen_global_seq`
(cursor for pull), `last_pushed_seq`, and `dirty` (local changes pending
push).

_Aavoid_: SyncCursor, sync position

### Update

A single entry in the server's append-only change log. Contains a
monotonic `global_seq` (server-assigned), the `doc_id` and `device_id`,
and an opaque encrypted blob. The server never inspects the blob.

_Aavoid_: Change, mutation, delta

### Settings

Per-device configuration that is not synced across devices. Different
devices may have different preferences (theme, font size, default folder,
sort order).

_Avoid_: Preferences, config

---

## Resolved Decisions

### Title is user-owned metadata

The title is never derived from markdown content or list items. New markdown
and list notes start with neutral `Untitled` and `List` defaults respectively;
after that, only an explicit title edit changes the title. Adding, editing, or
removing content must preserve it. An explicitly blank title resets to the
neutral default for that note type.

The title is stored in the Loro document at `metadata.title` so renames sync
with the note across devices. It is also mirrored in the SQLite `title` column
for FTS5 indexing and fast listing. The editor sends a title override only when
the user changed the title field; `null` means preserve the stored title.

### List notes are a first-class NoteType

The codebase supports `NoteType::List` alongside `NoteType::Markdown`.
List notes use a Loro list container (`doc.get_list("items")`) and have
dedicated operations (`list_add_item`, `list_remove_item`). The domain
model should be updated to reflect this. The `WrongNoteType` error variant
guards type-specific operations.

### Settings are stored globally (not per-device) in v1

_domain-model.md_ specifies per-device settings scoped by `device_id`.
The current SQLite schema has a simple `key TEXT PK, value TEXT` table
with no device scoping. Multi-device settings are deferred — acceptable
because the app is single-device in v1, but the schema must be migrated
before multi-device ships.

### Folder and tag names are globally unique (not per-account) in v1

The SQLite schema uses `UNIQUE` on `folders.name` and `tags.name` without
an account scope. This is a v1 simplification — acceptable because there's
one account. Must be migrated to per-account uniqueness before
multi-account support.

### sort_order column exists but is unused

The `notes.sort_order` column exists in the schema but the code hardcodes
`updated_at DESC` for listing. Manual sort ordering is deferred to a
later phase.

### Folders are referenced by ID, not nested

Notes reference a folder by `folder_id` (nullable). Deleting a folder
sets contained notes' `folder_id` to null (notes go to root, not deleted).
No trash/recycle bin in v1.
