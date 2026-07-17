use no_nonsense_notes_server::storage::Database;
use std::sync::Arc;

pub fn test_db() -> Arc<Database> {
    Arc::new(Database::open_in_memory().unwrap())
}
