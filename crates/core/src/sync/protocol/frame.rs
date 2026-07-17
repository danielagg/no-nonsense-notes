use super::*;

pub fn encode_push_frame(payload: &PushPayload) -> Vec<u8> {
    let sync_blob = encode_sync_blob(payload.note_type, payload.loro_blob);
    encode_raw_push_frame(payload.doc_id, payload.device_id, &sync_blob)
}

pub fn encode_delete_frame(doc_id: Uuid, device_id: Uuid) -> Vec<u8> {
    encode_raw_push_frame(doc_id, device_id, &encode_tombstone_blob())
}

fn encode_raw_push_frame(doc_id: Uuid, device_id: Uuid, sync_blob: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(2 + 16 + 16 + 4 + sync_blob.len());
    buf.push(VERSION);
    buf.push(MSG_PUSH);
    buf.extend_from_slice(doc_id.as_bytes());
    buf.extend_from_slice(device_id.as_bytes());
    buf.extend_from_slice(&(sync_blob.len() as u32).to_le_bytes());
    buf.extend_from_slice(&sync_blob);
    buf
}

pub fn decode_push_response(data: &[u8]) -> Result<i64, String> {
    if data.len() < 8 {
        return Err("push response too short".to_string());
    }
    let seq = i64::from_le_bytes(data[0..8].try_into().unwrap());
    Ok(seq)
}

pub fn encode_pull_request(last_seq: i64) -> String {
    format!("pull:{last_seq}")
}
