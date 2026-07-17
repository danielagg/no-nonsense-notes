use super::*;

pub fn encode_sync_blob(note_type: NoteType, loro_blob: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(1 + loro_blob.len());
    buf.push(note_type.as_byte());
    buf.extend_from_slice(loro_blob);
    buf
}

pub fn encode_tombstone_blob() -> Vec<u8> {
    vec![TOMBSTONE]
}

pub fn decode_sync_blob(data: &[u8]) -> Result<SyncPayload<'_>, String> {
    if data.is_empty() {
        return Err("sync blob too short".to_string());
    }
    if data[0] == TOMBSTONE {
        if data.len() != 1 {
            return Err("tombstone blob has trailing bytes".to_string());
        }
        return Ok(SyncPayload::Tombstone);
    }
    let note_type = NoteType::from_byte(data[0])
        .ok_or_else(|| format!("invalid note type byte: {}", data[0]))?;
    Ok(SyncPayload::Note {
        note_type,
        loro_blob: &data[1..],
    })
}
