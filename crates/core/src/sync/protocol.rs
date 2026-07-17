use base64::Engine;
use uuid::Uuid;

use crate::note::NoteType;

pub const VERSION: u8 = 1;
pub const MSG_PUSH: u8 = 0x01;
pub const TOMBSTONE: u8 = 0xFF;

pub const PROTOCOL_VERSION: u8 = 1;

pub struct PushPayload<'a> {
    pub doc_id: Uuid,
    pub device_id: Uuid,
    pub note_type: NoteType,
    pub loro_blob: &'a [u8],
}

pub struct PullEntry {
    pub doc_id: Uuid,
    pub payload: PullPayload,
}

#[derive(Debug, PartialEq)]
pub enum SyncPayload<'a> {
    Note {
        note_type: NoteType,
        loro_blob: &'a [u8],
    },
    Tombstone,
}

#[derive(Debug, PartialEq)]
pub enum PullPayload {
    Note {
        note_type: NoteType,
        loro_blob: Vec<u8>,
    },
    Tombstone,
}

pub struct PullResponse {
    pub current_seq: i64,
    pub entries: Vec<PullEntry>,
}

mod blob;
mod frame;
mod pull;

pub use blob::*;
pub use frame::*;
pub use pull::*;

#[cfg(test)]
mod tests;
