CREATE TABLE pending_sync (
    doc_id      TEXT PRIMARY KEY,
    note_type   TEXT NOT NULL,
    generation  INTEGER NOT NULL DEFAULT 1
);
