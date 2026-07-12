use std::env;
use std::fs;
use std::path::Path;

/// A single migration, either compiled into the crate via `include!`
/// or constructed at runtime.
pub struct Migration {
    pub version: i64,
    pub description: &'static str,
    pub sql: &'static str,
}

/// Applies all pending migrations in version order.
/// Returns the final schema version.
pub fn run(conn: &rusqlite::Connection, migrations: &[Migration]) -> Result<i64, rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _schema_version (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for m in migrations {
        if m.version <= current {
            continue;
        }
        conn.execute_batch(m.sql)?;
        conn.execute(
            "INSERT INTO _schema_version (version, description) VALUES (?1, ?2)",
            rusqlite::params![m.version, m.description],
        )?;
    }

    let final_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(final_version)
}

/// Scans `src/storage/migrations/*.sql` in the calling crate's manifest dir,
/// generates a Rust file in OUT_DIR containing the `MIGRATIONS` static array
/// and `MIGRATION_COUNT` constant.
///
/// Convention: filenames are `NNN_description.sql`
///   - version parsed from the numeric prefix
///   - description derived from the underscore-separated remainder
pub fn generate() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations_dir = Path::new(&manifest_dir).join("src/storage/migrations");

    println!("cargo:rerun-if-changed=src/storage/migrations");

    let mut entries: Vec<_> = fs::read_dir(&migrations_dir)
        .expect("failed to read migrations directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "sql"))
        .collect();

    entries.sort_by_key(|e| e.path());

    let count = entries.len();

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("migrations.rs");

    let mut content = String::from("static MIGRATIONS: &[Migration] = &[\n");
    for entry in &entries {
        let path = entry.path();
        let filename = path.file_stem().unwrap().to_str().unwrap();
        let version: i64 = filename
            .split('_')
            .next()
            .unwrap()
            .parse()
            .expect("migration filename must start with a number");
        let description: String = filename
            .split('_')
            .skip(1)
            .collect::<Vec<_>>()
            .join(" ");
        let abs_path = path.canonicalize().unwrap();
        let abs_str = abs_path.to_str().unwrap();
        content.push_str(&format!(
            r#"    Migration {{ version: {}, description: "{}", sql: include_str!("{}") }},"#,
            version, description, abs_str
        ));
        content.push('\n');
    }
    content.push_str("];\n");
    content.push_str(&format!("const MIGRATION_COUNT: usize = {};\n", count));

    fs::write(&dest_path, content).unwrap();
}
