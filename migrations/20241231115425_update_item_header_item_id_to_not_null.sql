CREATE TABLE IF NOT EXISTS item_header_new (item_id INTEGER NOT NULL REFERENCES item (id) ON DELETE CASCADE, name TEXT NOT NULL, value BLOB NOT NULL) STRICT;

INSERT INTO item_header_new (item_id, name, value) SELECT item_id, name, value FROM item_header;

DROP TABLE item_header;

ALTER TABLE item_header_new RENAME TO item_header;

CREATE INDEX IF NOT EXISTS idx_item_header_item_id ON item_header (item_id);
