# Phase one API contract

All JSON below uses snake_case. Errors: `{ "error": "readable message" }`.
Same-origin session cookie, HTTP-only; mutation requests must pass server Origin checks.
API base `/api/v1`. Public health GET `/healthz`.

- GET `/auth/status` -> `{setup_required:boolean,user:null|{id:string,username:string}}`
- POST `/auth/setup` body `{username,password}` -> `{user:{id,username}}`, creates first admin and signs in. Requires `X-Oxevu-Setup-Token` from the local `OXEVU_DATA_DIR/setup-token` file; show a token input during bootstrap and never persist it. Initial startup requires loopback binding. Password minimum is 12 bytes.
- POST `/auth/login` same body -> `{user:{id,username}}`
- POST `/auth/logout` -> 204
- GET `/libraries` -> `{libraries:[{id:string,name:string,path:string,kind:"movies"|"tv"}]}` (admin)
- POST `/libraries` body `{name,path,kind}` -> `{library:{id,name,path,kind}}` (admin)
- POST `/libraries/{id}/scan` -> `{scanned:number,errors:[string]}` (admin; synchronous bounded scan initially)
- GET `/items` -> `{items:[Item]}`
- GET `/items/{id}` -> `{item:Item}`
- GET `/items/{id}/stream` -> authenticated byte-range original file, MIME from actual container; 206/416 and HEAD supported.
- PUT `/items/{id}/progress` body `{position_seconds:number,completed:boolean}` -> 204

Item fields: `id:string,library_id:string,title:string,kind:"movie"|"episode",show_title:string|null,season_number:number|null,episode_number:number|null,duration_seconds:number|null,container:string|null,video_codec:string|null,audio_codec:string|null,position_seconds:number,completed:boolean`.
List items are playable movies/episodes. Frontend groups episodes into shows/seasons.
No artwork provider this phase; use honest artwork-free presentation, never fake catalog data.
No transcode this phase; clearly report browser playback failures and original delivery.
Frontend owns apps/web only; backend owns Cargo workspace/crates/server only. Parent owns shared docs/tooling and integration. Do not edit another owner's files. Report contract changes to parent. No Git init, commits, deployment or subdelegation.

Development: set backend `OXEVU_ORIGIN=http://localhost:5173` when using the Vite proxy. Production defaults to `http://localhost:3000`; the browser URL must match the configured origin.

## Artwork extension

Item adds nullable `poster_url`, `thumbnail_url`, `backdrop_url` (relative same-origin URLs).
GET `/items/{id}/artwork/{kind}` where kind is `poster`, `thumbnail`, or `backdrop`
requires the same session authorization as catalog/stream. Return proper image MIME
or 404. Discover local image sidecars during scan: movie stem-poster/backdrop/thumb,
movie folder poster/folder/backdrop, show ancestor poster/folder/backdrop and episode
stem-thumb. Show cards use episode's inherited show poster. Never attach a different
movie's artwork from a shared root. Canonicalize images and enforce library containment;
never serve arbitrary caller paths. Prefer existing art; generated thumbnails are cached
outside media roots, bounded by subprocess timeout/concurrency. No third-party catalog
provider or credentials required. Old catalog records without these fields remain valid.
