use super::super::*;
use crate::note::NoteId;

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
