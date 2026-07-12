use std::env;
use std::fs;
use std::path::Path;

/// Scans `src/storage/migrations/*.sql` in the calling crate's manifest dir,
/// generates a Rust file in OUT_DIR containing the `MIGRATIONS` static array.
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

    fs::write(&dest_path, content).unwrap();
}
