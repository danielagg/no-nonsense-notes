use super::*;

pub fn decode_pull_response(text: &str) -> Result<PullResponse, String> {
    let mut lines = text.lines();
    let first = lines.next().ok_or("empty pull response")?;
    let seq_str = first.strip_prefix("seq:").ok_or("expected seq: prefix")?;
    let current_seq: i64 = seq_str.parse().map_err(|_| "invalid seq number")?;

    let mut entries = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let colon_idx = line.find(':').ok_or("missing colon in pull entry")?;
        let doc_id_str = &line[..colon_idx];
        let blob_b64 = &line[colon_idx + 1..];
        let doc_id = Uuid::parse_str(doc_id_str).map_err(|e| format!("invalid doc_id: {e}"))?;
        let sync_blob = base64::engine::general_purpose::STANDARD
            .decode(blob_b64)
            .map_err(|e| format!("base64 decode: {e}"))?;
        let payload = match decode_sync_blob(&sync_blob)? {
            SyncPayload::Note {
                note_type,
                loro_blob,
            } => PullPayload::Note {
                note_type,
                loro_blob: loro_blob.to_vec(),
            },
            SyncPayload::Tombstone => PullPayload::Tombstone,
        };
        entries.push(PullEntry { doc_id, payload });
    }

    Ok(PullResponse {
        current_seq,
        entries,
    })
}
