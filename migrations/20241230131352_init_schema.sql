CREATE TABLE IF NOT EXISTS item (id INTEGER PRIMARY KEY AUTOINCREMENT, system TEXT, type TEXT, event_id INTEGER, entity_event_id INTEGER, user_agent TEXT, submit_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP) STRICT;
CREATE TABLE IF NOT EXISTS item_body (item_id INTEGER PRIMARY KEY REFERENCES item (id) ON DELETE CASCADE, body BLOB NOT NULL) STRICT;
CREATE TABLE IF NOT EXISTS item_header (item_id INTEGER REFERENCES item (id) ON DELETE CASCADE, name TEXT NOT NULL, value BLOB NOT NULL) STRICT;

CREATE INDEX IF NOT EXISTS idx_item_entity_event_id ON item (entity_event_id);
CREATE INDEX IF NOT EXISTS idx_item_event_id ON item (event_id);
CREATE INDEX IF NOT EXISTS idx_item_header_item_id ON item_header (item_id);
CREATE INDEX IF NOT EXISTS idx_item_submit_date ON item (submit_date);
CREATE INDEX IF NOT EXISTS idx_item_system ON item (system);
CREATE INDEX IF NOT EXISTS idx_item_type ON item (type);
CREATE INDEX IF NOT EXISTS idx_item_user_agent ON item (user_agent);
