mod records;
mod store;
mod sync;

pub use records::{NativeError, NoteKind, NoteRecord, SyncStatus};
pub use store::NotesStore;
pub use sync::{SyncDelegate, SyncSession};

uniffi::setup_scaffolding!();
