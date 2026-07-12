ALTER TABLE updates ADD COLUMN account_id TEXT NOT NULL DEFAULT '';

CREATE INDEX idx_updates_account_id ON updates(account_id);
