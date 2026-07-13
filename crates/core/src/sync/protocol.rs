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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::NoteId;

    #[test]
    fn encode_decode_sync_blob_round_trip() {
        let blob = vec![1, 2, 3, 4, 5];
        let encoded = encode_sync_blob(NoteType::Markdown, &blob);
        assert_eq!(
            decode_sync_blob(&encoded).unwrap(),
            SyncPayload::Note {
                note_type: NoteType::Markdown,
                loro_blob: &blob,
            }
        );
    }

    #[test]
    fn encode_decode_sync_blob_list() {
        let blob = vec![9, 9, 9];
        let encoded = encode_sync_blob(NoteType::List, &blob);
        assert_eq!(
            decode_sync_blob(&encoded).unwrap(),
            SyncPayload::Note {
                note_type: NoteType::List,
                loro_blob: &blob,
            }
        );
    }

    #[test]
    fn encode_decode_tombstone_round_trip() {
        let encoded = encode_tombstone_blob();
        assert_eq!(decode_sync_blob(&encoded).unwrap(), SyncPayload::Tombstone);
        assert!(decode_sync_blob(&[TOMBSTONE, 0]).is_err());
    }

    #[test]
    fn decode_sync_blob_empty_errors() {
        assert!(decode_sync_blob(&[]).is_err());
    }

    #[test]
    fn decode_sync_blob_invalid_note_type_errors() {
        assert!(decode_sync_blob(&[99]).is_err());
    }

    #[test]
    fn encode_push_frame_correct_format() {
        let doc_id = NoteId::now_v7();
        let device_id = NoteId::now_v7();
        let loro_blob = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let payload = PushPayload {
            doc_id,
            device_id,
            note_type: NoteType::Markdown,
            loro_blob: &loro_blob,
        };
        let frame = encode_push_frame(&payload);

        assert_eq!(frame[0], VERSION);
        assert_eq!(frame[1], MSG_PUSH);
        assert_eq!(&frame[2..18], doc_id.as_bytes());
        assert_eq!(&frame[18..34], device_id.as_bytes());
        let blob_len = u32::from_le_bytes(frame[34..38].try_into().unwrap());
        assert_eq!(blob_len as usize, 1 + loro_blob.len());
        assert_eq!(frame[38], NoteType::Markdown.as_byte());
        assert_eq!(&frame[39..], &loro_blob[..]);
    }

    #[test]
    fn encode_delete_frame_contains_tombstone() {
        let doc_id = NoteId::now_v7();
        let device_id = NoteId::now_v7();
        let frame = encode_delete_frame(doc_id, device_id);

        assert_eq!(u32::from_le_bytes(frame[34..38].try_into().unwrap()), 1);
        assert_eq!(&frame[38..], &[TOMBSTONE]);
    }

    #[test]
    fn decode_push_response_round_trip() {
        let seq = 42i64;
        let data = seq.to_le_bytes().to_vec();
        let decoded = decode_push_response(&data).unwrap();
        assert_eq!(decoded, seq);
    }

    #[test]
    fn decode_push_response_too_short() {
        assert!(decode_push_response(&[1, 2, 3]).is_err());
    }

    #[test]
    fn encode_pull_request_format() {
        let req = encode_pull_request(123);
        assert_eq!(req, "pull:123");
    }

    #[test]
    fn decode_pull_response_with_entries() {
        let doc_id = NoteId::now_v7();
        let loro_blob = vec![1, 2, 3];
        let sync_blob = encode_sync_blob(NoteType::List, &loro_blob);
        let b64 = base64::engine::general_purpose::STANDARD.encode(&sync_blob);
        let response = format!("seq:100\n{doc_id}:{b64}\n");

        let decoded = decode_pull_response(&response).unwrap();
        assert_eq!(decoded.current_seq, 100);
        assert_eq!(decoded.entries.len(), 1);
        assert_eq!(decoded.entries[0].doc_id, doc_id);
        assert_eq!(
            decoded.entries[0].payload,
            PullPayload::Note {
                note_type: NoteType::List,
                loro_blob,
            }
        );
    }

    #[test]
    fn decode_pull_response_empty() {
        let decoded = decode_pull_response("seq:50\n").unwrap();
        assert_eq!(decoded.current_seq, 50);
        assert!(decoded.entries.is_empty());
    }

    #[test]
    fn decode_pull_response_multiple_entries() {
        let doc1 = NoteId::now_v7();
        let doc2 = NoteId::now_v7();
        let blob1 = encode_sync_blob(NoteType::Markdown, &[1]);
        let blob2 = encode_sync_blob(NoteType::List, &[2, 3]);
        let b64_1 = base64::engine::general_purpose::STANDARD.encode(&blob1);
        let b64_2 = base64::engine::general_purpose::STANDARD.encode(&blob2);
        let response = format!("seq:200\n{doc1}:{b64_1}\n{doc2}:{b64_2}\n");

        let decoded = decode_pull_response(&response).unwrap();
        assert_eq!(decoded.entries.len(), 2);
        assert!(matches!(
            decoded.entries[0].payload,
            PullPayload::Note {
                note_type: NoteType::Markdown,
                ..
            }
        ));
        assert!(matches!(
            decoded.entries[1].payload,
            PullPayload::Note {
                note_type: NoteType::List,
                ..
            }
        ));
    }

    #[test]
    fn decode_pull_response_tombstone() {
        let doc_id = NoteId::now_v7();
        let b64 = base64::engine::general_purpose::STANDARD.encode(encode_tombstone_blob());
        let response = format!("seq:201\n{doc_id}:{b64}\n");

        let decoded = decode_pull_response(&response).unwrap();
        assert_eq!(decoded.entries[0].payload, PullPayload::Tombstone);
    }
}
