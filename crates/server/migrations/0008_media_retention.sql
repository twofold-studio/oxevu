CREATE TABLE item_reactions (
    user_id TEXT NOT NULL REFERENCES users(id),
    item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    value TEXT NOT NULL CHECK(value IN ('like','dislike')),
    PRIMARY KEY(user_id,item_id)
);
CREATE TABLE acquisition_removed_units (
    title_id TEXT NOT NULL REFERENCES acquisition_titles(id) ON DELETE CASCADE,
    season INTEGER NOT NULL,
    episode INTEGER NOT NULL,
    PRIMARY KEY(title_id,season,episode)
);
