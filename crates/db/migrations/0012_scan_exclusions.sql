-- Catalog-only cleanup must survive rescans without deleting media files.
CREATE TABLE scan_exclusions (
    path TEXT PRIMARY KEY,
    reason TEXT NOT NULL
);
