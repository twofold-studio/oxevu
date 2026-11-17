CREATE TABLE metadata_attempts (
    item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    attempted_at INTEGER NOT NULL,
    error TEXT NOT NULL
);

CREATE VIEW item_metadata_state AS
SELECT i.id, CASE
    WHEN json_extract(m.data, '$.metadata_source') = 'manual' THEN 1
    WHEN trim(COALESCE(CASE WHEN json_extract(i.data, '$.kind') = 'episode'
        THEN json_extract(m.data, '$.show_overview')
        ELSE json_extract(m.data, '$.overview') END, '')) NOT IN ('', 'N/A') THEN 1
    ELSE 0 END AS identified
FROM items i LEFT JOIN item_metadata m ON m.item_id = i.id;
