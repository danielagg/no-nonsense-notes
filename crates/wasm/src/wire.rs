use super::*;

#[wasm_bindgen(js_name = encodePushFrame)]
pub fn encode_push_frame(
    doc_id: &str,
    device_id: &str,
    note_type: &str,
    loro_blob: &[u8],
) -> JsResult<Vec<u8>> {
    let doc_id = doc_id
        .parse::<NoteId>()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let device_id = device_id
        .parse::<NoteId>()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let nt: NoteType = note_type
        .parse()
        .map_err(|e: String| JsValue::from_str(&e))?;

    let payload = protocol::PushPayload {
        doc_id,
        device_id,
        note_type: nt,
        loro_blob,
    };
    Ok(protocol::encode_push_frame(&payload))
}

#[wasm_bindgen(js_name = encodeDeleteFrame)]
pub fn encode_delete_frame(doc_id: &str, device_id: &str) -> JsResult<Vec<u8>> {
    let doc_id = doc_id
        .parse::<NoteId>()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let device_id = device_id
        .parse::<NoteId>()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(protocol::encode_delete_frame(doc_id, device_id))
}

#[wasm_bindgen(js_name = decodePushResponse)]
pub fn decode_push_response(data: &[u8]) -> JsResult<i64> {
    protocol::decode_push_response(data).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen(js_name = encodePullRequest)]
pub fn encode_pull_request(last_seq: i64) -> String {
    protocol::encode_pull_request(last_seq)
}

#[wasm_bindgen(js_name = decodePullResponse)]
pub fn decode_pull_response(text: &str) -> JsResult<JsValue> {
    let response = protocol::decode_pull_response(text).map_err(|e| JsValue::from_str(&e))?;

    let obj = js_sys::Object::new();
    set_field(
        &obj,
        "currentSeq",
        &JsValue::from_f64(response.current_seq as f64),
    )?;

    let entries = js_sys::Array::new();
    for entry in &response.entries {
        let entry_obj = js_sys::Object::new();
        set_field(
            &entry_obj,
            "docId",
            &JsValue::from_str(&entry.doc_id.to_string()),
        )?;
        match &entry.payload {
            protocol::PullPayload::Note {
                note_type,
                loro_blob,
            } => {
                set_field(&entry_obj, "deleted", &JsValue::FALSE)?;
                set_field(
                    &entry_obj,
                    "noteType",
                    &JsValue::from_str(note_type.as_str()),
                )?;
                set_field(
                    &entry_obj,
                    "loroBlob",
                    &js_sys::Uint8Array::from(&loro_blob[..]),
                )?;
            }
            protocol::PullPayload::Tombstone => {
                set_field(&entry_obj, "deleted", &JsValue::TRUE)?;
                set_field(&entry_obj, "noteType", &JsValue::NULL)?;
                set_field(
                    &entry_obj,
                    "loroBlob",
                    &js_sys::Uint8Array::new_with_length(0),
                )?;
            }
        }
        entries.push(&entry_obj);
    }
    set_field(&obj, "entries", &entries)?;

    Ok(obj.into())
}
