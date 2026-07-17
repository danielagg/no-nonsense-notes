use super::super::*;
use crate::note::NoteId;

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
