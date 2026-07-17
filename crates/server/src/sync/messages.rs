use rusqlite::params;

use crate::storage::Database;

/// Binary message format:
/// [version:1][type:1][payload:N]
///
/// Message types:
/// 0x01 = Push updates (client -> server)
/// 0x02 = Pull response (server -> client)
///
/// Push payload:
/// [doc_id:16 bytes UUID][device_id:16 bytes UUID][blob_len:4 bytes LE][blob:N]
///
/// Push response: [global_seq:8 bytes LE]
///
/// Pull request (text for now): "pull:<last_seq>"
/// Pull response payload (text for now):
/// seq:<current_seq>\n<doc_id>:<base64_blob>\n...
const VERSION: u8 = 1;
const MSG_PUSH: u8 = 0x01;

pub(super) fn handle_binary_message(
    db: &Database,
    account_id: &str,
    data: &[u8],
) -> anyhow::Result<(Vec<u8>, i64)> {
    if data.len() < 2 {
        anyhow::bail!("message too short");
    }

    let version = data[0];
    let msg_type = data[1];

    if version != VERSION {
        anyhow::bail!("unsupported version: {}", version);
    }

    match msg_type {
        MSG_PUSH => handle_push(db, account_id, &data[2..]),
        _ => anyhow::bail!("unknown message type: {}", msg_type),
    }
}

fn handle_push(db: &Database, account_id: &str, payload: &[u8]) -> anyhow::Result<(Vec<u8>, i64)> {
    if payload.len() < 36 {
        anyhow::bail!("push payload too short");
    }

    let doc_id = uuid::Uuid::from_slice(&payload[0..16])?;
    let device_id = uuid::Uuid::from_slice(&payload[16..32])?;
    let blob_len =
        u32::from_le_bytes([payload[32], payload[33], payload[34], payload[35]]) as usize;

    if payload.len() < 36 + blob_len {
        anyhow::bail!("blob length mismatch");
    }

    let blob = &payload[36..36 + blob_len];

    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO updates (doc_id, device_id, account_id, blob) VALUES (?1, ?2, ?3, ?4)",
        params![doc_id.to_string(), device_id.to_string(), account_id, blob],
    )?;

    let global_seq: i64 = conn.last_insert_rowid();

    let mut response = Vec::with_capacity(8);
    response.extend_from_slice(&global_seq.to_le_bytes());
    Ok((response, global_seq))
}

pub(super) fn handle_text_message(
    db: &Database,
    account_id: &str,
    text: &str,
) -> anyhow::Result<String> {
    let parts: Vec<&str> = text.splitn(2, ':').collect();
    if parts.len() != 2 || parts[0] != "pull" {
        anyhow::bail!("invalid command: {}", text);
    }

    let last_seq: i64 = parts[1]
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid seq"))?;

    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT doc_id, global_seq, blob FROM updates WHERE global_seq > ?1 AND account_id = ?2 ORDER BY global_seq ASC LIMIT 1000",
    )?;

    let entries: Vec<(String, i64, Vec<u8>)> = stmt
        .query_map(params![last_seq, account_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let current_seq = entries.iter().map(|e| e.1).max().unwrap_or(last_seq);

    let mut response = format!("seq:{}\n", current_seq);
    for (doc_id, _seq, blob) in &entries {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(blob);
        response.push_str(&format!("{}:{}\n", doc_id, encoded));
    }

    Ok(response)
}
