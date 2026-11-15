# Playback, administration and watch-party contract

Implementation contract for this phase; not a claim that endpoints already exist.
All below are under `/api/v1`, session-cookie authenticated, snake_case JSON.
Existing Origin protection applies to every mutation. Errors remain `{error:string}`.
Backend owns Cargo/crates; frontend owns apps/web; parent owns shared docs/integration.

## Users and authorization

Auth status/login/setup User adds `is_admin`, `disabled`, `can_play`,
`can_transcode`, `syncplay_access` (`create_join`|`join`|`disabled`).
Existing accounts migrate to admin; newly created accounts default non-admin.
User settings resource additionally has `library_ids:string[]` and
`all_libraries:boolean`. Admin bypasses library ACLs. Disabled revokes access.

- GET `/users` -> `{users:UserSettings[]}` admin only.
- POST `/users` -> `{user:UserSettings}` body username,password,is_admin,disabled,
  can_play,can_transcode,syncplay_access,all_libraries,library_ids.
- PUT `/users/{id}` same fields, password optional (omit to retain); admin only.
- PUT `/auth/password` body `{current_password,new_password}` -> 204; own account.

Protect last enabled admin. Role/access changes revoke affected sessions. Reject
unknown library grants. Enforce ACLs on catalog, item, original/artwork, progress,
playback manifests/segments and party membership, not only navigation. No public signup.
Keep failed-login throttling bounded, passwords hashed, sessions per user, logout
revokes current session. Never expose hashes or filesystem paths to nonadmins.

## Libraries

Existing library object retains `path` (first root for old clients), adds
`paths:string[]`, `enabled:boolean`. Creation/update body `{name,kind,paths,enabled}`;
accept legacy `path` creation. Movies and TV only. GET `/libraries` admin only.
PUT `/libraries/{id}` edits settings; POST `/libraries/{id}/remove` removes catalog
configuration only, never media (UI confirms). Existing POST scan retained; scans
must reconcile enabled roots while keeping offline roots safe. Preserve IDs and
progress for files that remain. Catalog grouping merges same named show across
roots of ONE library. Multiple library definitions remain distinct.
Validate canonical roots, reject overlaps and source/runtime/cache roots that
violate data separation. Read-only scanning. Admin-only POST
`/libraries/paths/suggest` accepts `{path}` and returns `{paths,truncated}` with
directory suggestions (maximum 20); virtual/system roots are excluded.
Manual scan now; file watching and scheduled scans may follow (do not show fake toggles).

## Server settings

GET `/settings` -> `{settings:Settings,capabilities:Capabilities}` admin only.
PUT `/settings` body complete Settings -> same envelope; validate and persist.
GET `/playback/capabilities` -> `{capabilities:Capabilities}` authorized users.

Settings = {
 transcoding_enabled:boolean, hardware_acceleration:"none"|"nvenc",
 hardware_encoding:boolean, hardware_decoding:boolean, gpu_index:number,
 allow_cpu_fallback:boolean, max_concurrent_transcodes:number,
 max_video_bitrate_mbps:number, max_height:number, cpu_crf:number,
 scratch_limit_gb:number, session_idle_seconds:number,
 cache_enabled:boolean, cache_limit_gb:number,
 resume_min_percent:number, resume_max_percent:number,
 resume_min_duration_seconds:number
}.

Initial defaults: enabled true, nvenc, hardware encoding true, hardware decoding
false, gpu 0, CPU fallback true, max jobs 2, 8 Mbps, height1080, CRF23,
scratch10GB, cache enabled/5GB, idle120s, resume5%/90%/300s. Cache range is
0.1–100GB. Ranges validated server side; settings
apply to new sessions, active sessions retain their plan. Recheck permissions live.
FFmpeg executable and scratch path are server env config (`OXEVU_FFMPEG`,
`OXEVU_TRANSCODE_DIR`), shown read-only in capabilities; scratch must remain outside
media roots. Fixed H264/AAC output, bounded threads and segment/process cleanup.
Capabilities = {ffmpeg_available:boolean,nvenc_available:boolean,
 hardware_decode_available:boolean,ffmpeg_path:string|null,scratch_path:string|null,
 notes:string[]}.
Capability claims require actual runtime probe or explicit unavailable reason.
Tone mapping, per-codec decode toggles, HEVC/AV1 output, advanced throttle,
trickplay and arbitrary FFmpeg arguments are deferred; no nonfunctional settings.
HDR must be detected/reported; do not silently claim correct HDR conversion.

## Playback sessions

POST `/playback/sessions` body
`{item_id,start_seconds:number,mode:"auto"|"direct"|"transcode",
direct_play_supported:boolean,max_height?:number,video_bitrate_mbps?:number}` -> `{session:PlaybackSession}`.
Browser assesses native container/codecs; auto chooses original when supported,
otherwise remux H264 compatible sources where possible, else H264/AAC HLS.
Permissions prohibit transcoding/remux if user lacks can_transcode.

PlaybackSession = {id,item_id,method:"direct"|"remux"|"cpu"|"nvenc",
status:"starting"|"ready"|"failed",stream_url:string|null,
start_seconds:number,duration_seconds:number|null,reason:string|null,
error:string|null,decoder?:"cuda"|"cpu"|"copy",
width:number|null,height:number|null,cache_hit:boolean,cache_end_seconds:number|null}.
GET `/playback/sessions/{id}` -> same envelope; owners only (admin may inspect).
POST `/playback/sessions/{id}/heartbeat` -> 204, renew idle lease.
POST `/playback/sessions/{id}/stop` -> 204; kill/reap children and clean scratch.
GET `/playback/sessions/{id}/hls/{file}` serves authorized manifest/segments only;
validate basenames and canonical ownership, never arbitrary paths. Stream URL points
to original endpoint for direct or HLS manifest otherwise. No secrets in URLs/logs.

For HLS start offset, media currentTime is relative to session.start_seconds.
Client custom timeline uses full item duration; recreate session at requested
absolute time when seeking outside available HLS window. Progress records absolute
position. Use hls.js where native HLS absent; clean up session on leave/unload,
heartbeat while active, no overlapping leaked sessions during rapid seeks.
Cached partial sessions end at `cache_end_seconds`; clients continue with a new
session there when the title continues. See [conversion cache](playback-cache.md).
Session creation includes `cached_ranges:[{start,end}]` in absolute seconds for
compatible cache entries. This is a snapshot, refreshed when a session opens;
cache eviction can invalidate it while playback remains open. Browser buffered
ranges are drawn separately at their actual offsets.

PUT `/items/{id}/favorite` accepts `{favorite:boolean}` and changes only the
authenticated user's favorite state. Catalog items include `favorite:boolean`.
The existing progress endpoint also supports explicit watched/unwatched changes.
Monitor global job/disk limits, timeout failed startup, cap stderr, clean idle and
shutdown. If GPU fails and fallback allowed retry CPU once and report actual method.

## Watch parties

Authenticated server-authoritative timeline; membership can never grant library access.
GET `/parties` -> `{parties:Party[]}` (only parties caller may join/access).
POST `/parties` body `{name,item_id}` -> `{party:Party}`; requires create_join.
POST `/parties/{id}/join` -> same; requires join/create_join and item access.
POST `/parties/{id}/leave` -> 204; host leaving closes party (clear UI).
GET `/parties/{id}` -> `{party:Party}` members only.
POST `/parties/{id}/control` body
`{action:"play"|"pause"|"seek"|"item",position_seconds?:number,item_id?:string}`
-> same; host only; validate item for EVERY member before change, else reject.

Party = {id,name,host_user_id,item_id,paused:boolean,position_seconds:number,
updated_at_ms:number,server_now_ms:number,revision:number,
members:[{user_id,username}]}.
Use polling 1s first (no websocket dependency required); snapshots extrapolate
playing position via server clock, estimate clock offset/RTT. Ignore stale revisions,
avoid event feedback loops and microseek jitter, correct large drift; follower
controls follow host. Autoplay blocked state must offer explicit Join playback click.
Each member gets independent authorized playback session; maintain progress separately.
Host controls entire timeline, including pause and item changes. Leave/close cleans
client sessions; disabled users lose membership/access. Bound rooms/members and
clean idle rooms, make buffering/reconnect visible without pretending perfect sync.

## Acceptance

Migration of existing user/library data, two-user ACL separation and admin controls;
CPU and NVIDIA conversion on vd using samples; GPU failure fallback; seeking/resume,
manifest/segment auth, job/disk bounds and cleanup; settings persist across restart;
two browser party participants synchronize play/pause/seek and cannot control as guest.
Document measured/untested limits; don't call this phase complete from mocks alone.

## Subtitles and integrations

GET `/items/{id}/subtitles` lists embedded text/image tracks, matching sidecars
and downloaded subtitles. GET `/items/{id}/subtitles/{track}` serves authenticated
WebVTT for supported text formats. Bitmap tracks are listed as unsupported.
The browser subtracts the HLS session offset from cue times.

Admin GET/PUT `/integrations` reads redacted provider state and updates optional
TMDB, OMDb, OpenSubtitles and keyless studio-image configuration. Blank secrets
preserve saved values; `clear_credentials:true` removes them. See
[integration setup](integrations.md) for provider requirements and attribution.

Admin GET `/items/{id}/metadata/search?provider=tmdb&query=...` searches candidates;
POST `/items/{id}/metadata/match` with `{provider,id}` applies a confirmed match.
GET `/items/{id}/subtitles/search?language=en` searches OpenSubtitles; POST
`/items/{id}/subtitles/download` with `{id,language}` caches a selected subtitle.
All item routes enforce library access; downloads also require conversion permission.
