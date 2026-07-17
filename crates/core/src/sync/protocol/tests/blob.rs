use super::super::*;
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
