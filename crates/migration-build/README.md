# Migration build crate

`migration-build` is a private workspace helper shared by the core and server
crates.

At build time it discovers numbered SQLite migration files and generates the
static migration list consumed by each crate. At runtime it applies migrations
in version order and records them in `_schema_version`.

Migration files follow this convention:

```text
001_initial.sql
002_add_note_type.sql
```

The core and server own separate migration directories and schemas; this crate
only supplies the common mechanism.

## Check

```sh
cargo test -p migration-build
```
