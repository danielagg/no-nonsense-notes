use super::super::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn encode_push_frame_produces_valid_frame() {
    let doc_id = NoteId::now_v7().to_string();
    let device_id = NoteId::now_v7().to_string();
    let blob = vec![0xDE, 0xAD, 0xBE, 0xEF];

    let frame = encode_push_frame(&doc_id, &device_id, "markdown", &blob).unwrap();
    assert!(frame.len() > 38);
    assert_eq!(frame[0], 1);
    assert_eq!(frame[1], 1);
}

#[wasm_bindgen_test]
fn encode_delete_frame_produces_tombstone_frame() {
    let doc_id = NoteId::now_v7().to_string();
    let device_id = NoteId::now_v7().to_string();

    let frame = encode_delete_frame(&doc_id, &device_id).unwrap();
    assert_eq!(u32::from_le_bytes(frame[34..38].try_into().unwrap()), 1);
    assert_eq!(frame[38], protocol::TOMBSTONE);
}

#[wasm_bindgen_test]
fn decode_pull_response_exposes_tombstone() {
    let doc_id = NoteId::now_v7();
    let response_text = format!("seq:51\n{doc_id}:/w==\n");
    let response = decode_pull_response(&response_text).unwrap();
    let entries: js_sys::Array = js_sys::Reflect::get(&response, &JsValue::from_str("entries"))
        .unwrap()
        .into();
    let entry = entries.get(0);

    assert_eq!(
        js_sys::Reflect::get(&entry, &JsValue::from_str("deleted"))
            .unwrap()
            .as_bool(),
        Some(true)
    );
    assert!(
        js_sys::Reflect::get(&entry, &JsValue::from_str("noteType"))
            .unwrap()
            .is_null()
    );
}

#[wasm_bindgen_test]
fn decode_push_response_round_trip() {
    let seq_bytes = 99i64.to_le_bytes();
    let seq = decode_push_response(&seq_bytes).unwrap();
    assert_eq!(seq, 99);
}

#[wasm_bindgen_test]
fn encode_pull_request_format() {
    let req = encode_pull_request(123);
    assert_eq!(req, "pull:123");
}

#[wasm_bindgen_test]
fn decode_pull_response_empty() {
    let response = decode_pull_response("seq:50\n").unwrap();
    let current_seq = js_sys::Reflect::get(&response, &JsValue::from_str("currentSeq"))
        .unwrap()
        .as_f64()
        .unwrap();
    assert_eq!(current_seq, 50.0);

    let entries = js_sys::Reflect::get(&response, &JsValue::from_str("entries")).unwrap();
    let arr: js_sys::Array = entries.into();
    assert_eq!(arr.length(), 0);
}
