use super::*;

pub(crate) fn set_field(obj: &js_sys::Object, key: &str, val: &JsValue) -> JsResult<()> {
    js_sys::Reflect::set(obj, &JsValue::from_str(key), val)?;
    Ok(())
}

pub(crate) fn note_to_js(note: &Note) -> JsResult<JsValue> {
    let obj = js_sys::Object::new();

    set_field(&obj, "id", &JsValue::from_str(&note.id.to_string()))?;
    set_field(
        &obj,
        "folderId",
        &note
            .folder_id
            .map(|f| JsValue::from_str(&f.to_string()))
            .unwrap_or(JsValue::NULL),
    )?;
    set_field(
        &obj,
        "noteType",
        &JsValue::from_str(note.note_type.as_str()),
    )?;
    set_field(&obj, "title", &JsValue::from_str(&note.title))?;
    set_field(
        &obj,
        "contentPlaintext",
        &JsValue::from_str(&note.content_plaintext),
    )?;
    set_field(
        &obj,
        "contentLoroBlob",
        &js_sys::Uint8Array::from(&note.content_loro_blob[..]),
    )?;
    set_field(
        &obj,
        "contentHash",
        &js_sys::Uint8Array::from(&note.content_hash[..]),
    )?;
    set_field(
        &obj,
        "createdAt",
        &JsValue::from_str(&note.created_at.to_rfc3339()),
    )?;
    set_field(
        &obj,
        "updatedAt",
        &JsValue::from_str(&note.updated_at.to_rfc3339()),
    )?;
    set_field(&obj, "isDeleted", &JsValue::from_bool(note.is_deleted))?;
    set_field(
        &obj,
        "deletedAt",
        &note
            .deleted_at
            .map(|d| JsValue::from_str(&d.to_rfc3339()))
            .unwrap_or(JsValue::NULL),
    )?;
    set_field(
        &obj,
        "sortOrder",
        &JsValue::from_f64(note.sort_order as f64),
    )?;

    Ok(obj.into())
}

pub(crate) fn notes_to_js(notes: &[Note]) -> JsResult<JsValue> {
    let arr = js_sys::Array::new();
    for note in notes {
        arr.push(&note_to_js(note)?);
    }
    Ok(arr.into())
}
