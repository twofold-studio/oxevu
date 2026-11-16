-- A stable library identity while its verified torrent source is still downloading.
CREATE TABLE acquisition_stream_items (
    item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    download_id TEXT NOT NULL REFERENCES acquisition_downloads(id) ON DELETE CASCADE,
    title_id TEXT NOT NULL REFERENCES acquisition_titles(id) ON DELETE CASCADE,
    file_index INTEGER NOT NULL CHECK(file_index >= 0),
    UNIQUE(download_id, file_index)
);
