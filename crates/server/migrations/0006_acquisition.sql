CREATE TABLE acquisition_titles(
 id TEXT PRIMARY KEY, kind TEXT NOT NULL CHECK(kind IN ('movie','tv')),
 provider_id INTEGER NOT NULL CHECK(provider_id>0), data TEXT NOT NULL,
 refreshed_at INTEGER NOT NULL DEFAULT(unixepoch()), UNIQUE(kind,provider_id)
);
CREATE TABLE acquisition_episodes(
 title_id TEXT NOT NULL REFERENCES acquisition_titles(id), season INTEGER NOT NULL,
 episode INTEGER NOT NULL, data TEXT NOT NULL, air_date TEXT,
 PRIMARY KEY(title_id,season,episode)
);
CREATE TABLE acquisition_permissions(
 user_id TEXT PRIMARY KEY REFERENCES users(id), can_request BOOLEAN NOT NULL DEFAULT 1,
 auto_approve BOOLEAN NOT NULL DEFAULT 0, manage_requests BOOLEAN NOT NULL DEFAULT 0,
 manage_downloads BOOLEAN NOT NULL DEFAULT 0, manage_indexers BOOLEAN NOT NULL DEFAULT 0,
 request_limit INTEGER NOT NULL DEFAULT 10 CHECK(request_limit BETWEEN 0 AND 1000)
);
CREATE TABLE acquisition_requests(
 id TEXT PRIMARY KEY, title_id TEXT NOT NULL REFERENCES acquisition_titles(id),
 user_id TEXT NOT NULL REFERENCES users(id), scope TEXT NOT NULL,
 status TEXT NOT NULL CHECK(status IN ('pending','approved','fulfilled','declined','cancelled')),
 note TEXT NOT NULL DEFAULT '', created_at INTEGER NOT NULL DEFAULT(unixepoch()),
 updated_at INTEGER NOT NULL DEFAULT(unixepoch())
);
CREATE UNIQUE INDEX acquisition_active_request ON acquisition_requests(title_id,user_id,scope)
 WHERE status IN ('pending','approved','fulfilled');
CREATE TABLE acquisition_monitors(
 title_id TEXT PRIMARY KEY REFERENCES acquisition_titles(id), enabled BOOLEAN NOT NULL DEFAULT 1,
 scope TEXT NOT NULL, library_id TEXT REFERENCES libraries(id) ON DELETE SET NULL, root TEXT,
 profile_id TEXT, automatic BOOLEAN NOT NULL DEFAULT 0, root_device INTEGER,
 next_search INTEGER NOT NULL DEFAULT 0, failures INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE acquisition_config(id INTEGER PRIMARY KEY CHECK(id=1), data TEXT NOT NULL);
CREATE TABLE acquisition_profiles(id TEXT PRIMARY KEY, name TEXT NOT NULL, data TEXT NOT NULL);
CREATE TABLE acquisition_indexers(id TEXT PRIMARY KEY, name TEXT NOT NULL, data TEXT NOT NULL);
CREATE TABLE acquisition_feeds(indexer_id TEXT PRIMARY KEY REFERENCES acquisition_indexers(id),next_poll INTEGER NOT NULL DEFAULT 0);
CREATE TABLE acquisition_releases(
 id TEXT PRIMARY KEY, title_id TEXT NOT NULL REFERENCES acquisition_titles(id),
 indexer_id TEXT REFERENCES acquisition_indexers(id), data TEXT NOT NULL,
 created_at INTEGER NOT NULL DEFAULT(unixepoch()), blocked_until INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE acquisition_downloads(
 id TEXT PRIMARY KEY, title_id TEXT REFERENCES acquisition_titles(id),
 release_id TEXT REFERENCES acquisition_releases(id), infohash TEXT UNIQUE,
 engine_id INTEGER, state TEXT NOT NULL DEFAULT 'queued', data TEXT NOT NULL,
 error TEXT, created_at INTEGER NOT NULL DEFAULT(unixepoch()),
 updated_at INTEGER NOT NULL DEFAULT(unixepoch()), seed_seconds INTEGER NOT NULL DEFAULT 0,
 retry_at INTEGER NOT NULL DEFAULT 0, attempts INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE acquisition_imports(
 id TEXT PRIMARY KEY, download_id TEXT NOT NULL REFERENCES acquisition_downloads(id),
 file_index INTEGER NOT NULL, title_id TEXT NOT NULL REFERENCES acquisition_titles(id),
 season INTEGER NOT NULL DEFAULT -1, episode INTEGER NOT NULL DEFAULT -1,
 destination TEXT NOT NULL UNIQUE, source TEXT NOT NULL, checksum TEXT,
 state TEXT NOT NULL DEFAULT 'planned', error TEXT, item_id TEXT, coverage TEXT NOT NULL DEFAULT '[]',
 library_id TEXT REFERENCES libraries(id) ON DELETE SET NULL, plan TEXT NOT NULL DEFAULT '{}',
 UNIQUE(download_id,file_index,season,episode)
);
CREATE TABLE acquisition_events(
 id INTEGER PRIMARY KEY AUTOINCREMENT, entity_id TEXT NOT NULL,
 message TEXT NOT NULL, created_at INTEGER NOT NULL DEFAULT(unixepoch())
);
CREATE INDEX acquisition_request_owner ON acquisition_requests(user_id,status);
CREATE INDEX acquisition_download_state ON acquisition_downloads(state,retry_at);
CREATE TABLE acquisition_migrations(source_key TEXT PRIMARY KEY,title_id TEXT NOT NULL REFERENCES acquisition_titles(id),created_at INTEGER NOT NULL);
CREATE TABLE acquisition_claims(title_id TEXT NOT NULL REFERENCES acquisition_titles(id),season INTEGER NOT NULL,episode INTEGER NOT NULL,download_id TEXT NOT NULL REFERENCES acquisition_downloads(id),PRIMARY KEY(title_id,season,episode));
