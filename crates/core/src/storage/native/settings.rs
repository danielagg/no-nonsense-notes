use super::*;

impl NativeStore {
    pub fn cursor(&self) -> Result<i64, StorageError> {
        Ok(self
            .setting("sync_cursor")?
            .and_then(|value| value.parse().ok())
            .unwrap_or(0))
    }

    pub fn device_id(&self) -> Result<Uuid, StorageError> {
        if let Some(value) = self.setting("device_id")? {
            return value
                .parse()
                .map_err(|e: uuid::Error| StorageError::Parse(e.to_string()));
        }
        let id = Uuid::now_v7();
        self.set_setting("device_id", &id.to_string())?;
        Ok(id)
    }
    pub(super) fn setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        let database = self
            .database
            .lock()
            .map_err(|_| StorageError::Protocol("database lock poisoned".into()))?;
        Ok(database
            .connection()
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub(super) fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let database = self
            .database
            .lock()
            .map_err(|_| StorageError::Protocol("database lock poisoned".into()))?;
        database.connection().execute(
            "INSERT INTO settings(key, value) VALUES(?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}
