CREATE TABLE notes (
    id                  TEXT PRIMARY KEY,
    folder_id           TEXT REFERENCES folders(id),
    title               TEXT NOT NULL DEFAULT 'Untitled',
    content_plaintext   TEXT NOT NULL DEFAULT '',
    content_loro_blob   BLOB NOT NULL DEFAULT (x''),
    content_hash        BLOB NOT NULL DEFAULT (x''),
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    is_deleted          INTEGER NOT NULL DEFAULT 0,
    deleted_at          TEXT,
    sort_order          INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE folders (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL
);

CREATE TABLE tags (
    id      TEXT PRIMARY KEY,
    name    TEXT NOT NULL UNIQUE,
    color   TEXT
);

CREATE TABLE note_tags (
    note_id TEXT NOT NULL REFERENCES notes(id),
    tag_id  TEXT NOT NULL REFERENCES tags(id),
    PRIMARY KEY (note_id, tag_id)
);

CREATE TABLE settings (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);

CREATE VIRTUAL TABLE notes_fts USING fts5(
    title, content_plaintext
);
