CREATE TABLE user_access(user_id TEXT PRIMARY KEY REFERENCES users(id),is_admin BOOLEAN NOT NULL DEFAULT 0,disabled BOOLEAN NOT NULL DEFAULT 0,can_play BOOLEAN NOT NULL DEFAULT 1,can_transcode BOOLEAN NOT NULL DEFAULT 1,syncplay_access TEXT NOT NULL DEFAULT 'create_join',all_libraries BOOLEAN NOT NULL DEFAULT 1);
INSERT INTO user_access(user_id,is_admin) SELECT id,1 FROM users;
CREATE TABLE library_grants(user_id TEXT NOT NULL REFERENCES users(id),library_id TEXT NOT NULL REFERENCES libraries(id),PRIMARY KEY(user_id,library_id));
CREATE TABLE library_options(library_id TEXT PRIMARY KEY REFERENCES libraries(id),enabled BOOLEAN NOT NULL DEFAULT 1);
CREATE TABLE library_roots(library_id TEXT NOT NULL REFERENCES libraries(id),path TEXT NOT NULL UNIQUE,PRIMARY KEY(library_id,path));
INSERT INTO library_roots SELECT id,path FROM libraries;
INSERT INTO library_options SELECT id,1 FROM libraries;
CREATE TABLE login_attempts(username TEXT PRIMARY KEY,attempts INTEGER NOT NULL,expires INTEGER NOT NULL);
