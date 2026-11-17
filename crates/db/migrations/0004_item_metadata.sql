CREATE TABLE item_metadata (
    item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    data TEXT NOT NULL
);
CREATE TABLE item_subtitles (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    language TEXT NOT NULL,
    file TEXT NOT NULL
);
