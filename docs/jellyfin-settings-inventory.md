# Jellyfin settings inventory

Reference for Oxevu's FFmpeg/NVIDIA settings, library roots, users/ACLs and
watch-party work. Observed read-only in the Jellyfin web dashboard on `vd`.

- **Observed:** 2026-11-15, Jellyfin server 12.4 (server, web and API client
  report 12.4.0, build "Release"). Bundled FFmpeg: `jellyfin-ffmpeg 8.1.3` from
  the Nix store.
- **Method:** DOM enumeration of each form (labels, control types, choices, help
  text). Dropdowns were opened to read choices; hardware-acceleration and access
  toggles were changed temporarily to reveal conditional fields, then discarded
  by navigating away. Nothing was saved.
- **Values:** "now" shows the current `vd` value where it helps; defaults are
  marked where the UI states them. No credentials, user names, IDs or device
  names are recorded.

Classification: **Now** = implement in the next phase; **Later** = worth doing
after the milestone; **Out** = out of scope or covered differently in Oxevu.

## Dashboard map

Server: Dashboard, General, Branding, Users, **Libraries** (Libraries, Display,
Metadata, NFO Settings), **Playback** (Transcoding, Resume, Streaming,
Trickplay). Devices: Devices, Activity. Live TV: Live TV, DVR. Plugins.
Advanced: Networking, API Keys, Backups, Logs, Scheduled Tasks. There is also a
user-facing Metadata Manager and per-user Settings (Profile, Quick Connect,
Display, Home, Playback, Subtitles, Controls).

## Playback → Transcoding

### Hardware acceleration

| Field | Control | Choices / range | Now on `vd` | Class |
|---|---|---|---|---|
| Hardware acceleration (`HardwareAccelerationType`) | select | None, AMD AMF, Nvidia NVENC, Intel Quicksync (QSV), VAAPI, Rockchip MPP, Apple VideoToolBox, V4L2 | NVENC | **Now**: None + NVENC only; others Later/Out |
| Enable hardware decoding for | checkbox per codec | H264, HEVC, MPEG2, MPEG4, VC1, VP8, VP9, AV1 | all except MPEG4 | **Now** (NVDEC codec list) |
| 10-bit / RExt decode | checkbox | HEVC 10bit, VP9 10bit, HEVC RExt 8/10bit, HEVC RExt 12bit | all on | **Now** (single "allow 10-bit HW decode" may suffice); RExt Later |
| Enable enhanced NVDEC decoder | checkbox | help: disable to fall back to CUVID on decode errors | on | Later (pick one decoder path first) |
| Enable hardware encoding | checkbox | — | on | **Now** |
| Allow encoding in HEVC format | checkbox | — | off | Later |
| Allow encoding in AV1 format | checkbox | — | off | Later |

Conditional fields revealed by switching the type (not saved):

- **None:** hides decode codecs, HW encoding and the tone-mapping enable/mode.
- **NVENC:** shows all codecs plus the enhanced NVDEC toggle.
- **AMF:** no MPEG4/VP8/RExt/NVDEC toggles.
- **QSV:** `QSV Device` text (render node or Windows index), "Prefer OS native
  DXVA or VA-API hardware decoders", Intel Low-Power H.264/HEVC encoder
  toggles (help: needs i915 HuC firmware on Linux), "Enable VPP Tone mapping"
  with brightness gain (0–100, default 16) and contrast gain (1–2, default 1).
- **VAAPI:** `VA-API Device` (default `/dev/dri/renderD128`), the Intel
  Low-Power encoder toggles and VPP tone mapping; no prefer-native toggle.
- **RKMPP:** adds MPEG1 decode; drops VC1/RExt.
- **VideoToolbox:** reduced codec list, "Enable VideoToolbox Tone mapping".
- **V4L2:** only H264 decode; tone-mapping section hidden.

All non-NVIDIA backends: **Out** for now (single-host NVIDIA target); keep the
type field an enum so VAAPI/QSV can be added later.

### Tone mapping (HDR → SDR)

| Field | Control | Choices / range | Now | Class |
|---|---|---|---|---|
| Enable Tone mapping | checkbox | help: 10-bit HDR10, HLG, DoVi; needs GPGPU runtime (CUDA/OpenCL) | off | **Now** (on/off with fixed defaults) |
| Tone mapping algorithm | select | None, Clip, Linear, Gamma, Reinhard, Hable, Mobius, BT.2390 (recommended) | BT.2390 | Later (hard-code BT.2390) |
| Tone mapping mode | select | Auto, MAX, RGB, LUM, ITP | Auto | Later |
| Tone mapping range | select | Auto, TV, PC | Auto | Later |
| Tone mapping desat | number ≥0 | recommended 0 | 0 | Later |
| Tone mapping peak | number ≥0 | default 100 (1000 nit) | 100 | Later |
| Tone mapping param | number ≥0 | normally blank | blank | Out |

### Encoder, paths and process control

| Field | Control | Choices / range | Now | Class |
|---|---|---|---|---|
| Transcoding thread count (`EncodingThreadCount`) | select | Auto(-1), 1–16, Max(0) | Max | Later (fixed sensible default) |
| FFmpeg path | text | file or folder | Nix store path | **Now** (config file/env, not web-editable) |
| Transcode path | text | blank = server default | blank | **Now** as scratch dir under runtime data (invariant: outside media roots) |
| Fallback font folder path / Enable fallback fonts | text + checkbox | for client subtitle rendering | blank / off | Later |
| Enable VBR audio encoding | checkbox | — | off | Later |
| Audio boost when downmixing | number 0.5–3, step 0.1 | — | 2 | Later |
| Stereo downmix algorithm | select | None, Dave750, NightmodeDialogue, RFC7845, AC-4 | None | Later |
| Audio seek strategy (HLS) | select | TrimCopiedAudio, TranscodeAudio | TrimCopiedAudio | Later; note the seek/copy problem it solves |
| Max muxing queue size | text | recommended 2048 | 2048 | **Now** as a hard-coded FFmpeg arg |
| Encoding preset | select | Auto, veryslow … ultrafast | Auto | Later (map to NVENC `p1–p7`) |
| H.265 CRF / H.264 CRF | number 0–51 | defaults 28 / 23; software encoders only | 28 / 23 | Later (CPU fallback quality) |
| Deinterlacing method | select | YADIF, BWDIF | YADIF | Later (HW deinterlacer used with NVENC) |
| Double the frame rate when deinterlacing | checkbox | bob deinterlacing | off | Later |
| Allow subtitle extraction on the fly | checkbox | else burn in unsupported subs | on | **Now** (text sub extraction to avoid transcodes) |
| Throttle Transcodes | checkbox | pause FFmpeg when far ahead | on | **Now** |
| Throttle after | number 10–3600 s | — | 180 | **Now** (fixed default; configurable later) |
| Delete segments | checkbox | delete after client download | on | **Now** (bounded scratch) |
| Time to keep segments | number 10–3600 s | — | 720 | **Now** (fixed default) |

## Playback → Resume

| Field | Control | Range | Now | Class |
|---|---|---|---|---|
| Minimum resume percentage | number 0–100 | unplayed if stopped before | 5 | **Now** |
| Maximum resume percentage | number 1–100 | played if stopped after | 90 | **Now** |
| Minimum resume duration | number ≥0 s | shortest video that stores position | 300 | **Now** |
| Minimum audiobook resume (min) | number 0–100 | — | 5 | Out (no audiobooks yet) |
| Audiobook remaining minutes to resume | number 1–100 | — | 5 | Out |

## Playback → Streaming

| Field | Control | Range | Now | Class |
|---|---|---|---|---|
| Internet streaming bitrate limit (Mbps) | number 0–2147.25, step 0.25 | per stream, out-of-network clients; blank = none | blank | Later (needs LAN/remote classification; per-user limit below) |

## Playback → Trickplay

| Field | Control | Choices / range | Now | Class |
|---|---|---|---|---|
| Enable hardware decoding | checkbox | — | off | Later |
| Enable hardware accelerated MJPEG encoding | checkbox | QSV, VA-API, VideoToolbox, RKMPP only | off | Out (no NVENC effect) |
| Only generate images from key frames | checkbox | faster, less accurate | off | Later |
| Scan behavior | select | Non Blocking, Blocking | Non Blocking | Later (Oxevu should stay non-blocking) |
| Process priority | select | High, Above Normal, Normal, Below Normal, Idle | Below Normal | Later |
| Image interval | number ≥1 ms | — | 10000 | Later |
| Width resolutions | text (CSV px) | — | 320 | Later |
| Tile width / tile height | number ≥1 | — | 10 / 10 | Later |
| JPEG quality | number 1–100 | — | 90 | Later |
| Qscale | number 2–31 | 2 best | 4 | Later |
| FFmpeg threads | number ≥0 | `-threads` | 1 | Later |

Trickplay as a whole is **Later**: useful scrubbing previews, but it is a
bounded background job with storage cost, not milestone playback.

## Libraries

### Library list (`/dashboard/libraries`)

Buttons: **Add Media Library**, **Scan All Libraries**. Each library card menu:
Edit images, Manage library, Rename, Scan library, Remove. `vd` has Collections,
Books, Movies (one folder) and Shows (one folder).

### Add Media Library / Manage library dialog

Header fields (add only): **Content type** select — Movies, Music, Shows, Books,
Home Videos and Photos, Music Videos, Mixed Movies and Shows; **Display name**
text. **Folders** list with an Add button that opens a "Select Path" server
directory browser (free-text Folder field, Refresh, list of server mounts).
Manage dialog warns that metadata/artwork changes only apply to new items until
a manual refresh. OK saves; Back discards.

Library settings by type (✓ = shown):

| Field | Control / choices | Movies | Shows | Mixed | Home video | Music video | Music | Books | Class |
|---|---|---|---|---|---|---|---|---|---|
| Enable the library | checkbox; hides from all users | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | **Now** |
| Preferred download language | select (ISO 639) | ✓ | ✓ | ✓ | | | ✓ | | Later (metadata) |
| Country/Region | select | ✓ | ✓ | ✓ | | | ✓ | | Later |
| Display the photos | checkbox | | | | ✓ | | | | Out |
| Special season display name | text, default "Specials" | | ✓ | | | | | | **Now** (constant "Specials") |
| Prefer embedded titles over filenames | checkbox | ✓ | ✓ | ✓ | ✓ | ✓ | | | Later |
| Prefer embedded titles for extras | checkbox | ✓ (manage) | ✓ (manage) | | | | | | Later |
| Prefer embedded episode info over filenames | checkbox | | ✓ | | | | | | Later |
| Disable different types of embedded subtitles | select: Allow All, Allow Text, Allow Image, Allow None; needs full refresh | ✓ | ✓ | ✓ | | ✓ | | | Later |
| Enable real time monitoring | checkbox; immediate processing on supported FS | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | **Now** (per-root watch toggle, rescan fallback) |
| Enable LUFS scan | checkbox | | | | | | ✓ | | Out |
| Automatically add to collection | checkbox; ≥2 movies with same collection name | ✓ | | ✓ | | | | | Later |
| Metadata downloaders (per item type, ordered) | checkbox + up/down | TMDb, OMDb | Shows: TMDb, OMDb; Seasons: TMDb; Episodes: TMDb, OMDb | union | | | MusicBrainz, TheAudioDB | readers: Comic, EPUB, OPF | Later |
| Automatically refresh metadata from the internet | select: Never, 30, 60, 90 days | ✓ | ✓ | ✓ | | | ✓ | | Later |
| Metadata savers | checkbox: Nfo | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | Out (writes to media; violates read-only default) |
| Image fetchers (per type, ordered) + Fetcher Settings | checkbox + up/down; dialog below | TMDb, OMDb, Embedded, Screen Grabber | per Show/Season/Episode | union | Embedded, Screen Grabber | Embedded, Screen Grabber | AudioDB, Image Extractor | CBZ cover, EPUB | Later |
| Similar item providers | checkbox | TMDb, Local Genre/Tag | same | same | | | ListenBrainz, Local | Local | Out |
| Save artwork into media folders | checkbox | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Out (keep artwork in runtime data) |
| Automatically merge series spread across folders | checkbox | | ✓ | | | | | | **Now** (always merge by show identity across roots) |
| Trickplay: enable / extract during scan / save next to media | checkbox ×3 | ✓ | ✓ | ✓ | ✓ | ✓ | | | Later; never save next to media |
| Chapter images: enable / extract during scan | checkbox ×2 | ✓ | ✓ | ✓ | ✓ | ✓ | | | Later |
| Subtitle downloads: languages, downloaders (Open Subtitles) | checkbox list | ✓ | ✓ | ✓ | | | | | Out (plugin/external service) |
| Only perfect-match subtitles; skip if audio matches; skip if embedded subs; save subtitles into media folders | checkbox ×4 | ✓ | ✓ | ✓ | | save only | | | Out |
| Lyrics: save into media folders | checkbox | | | | | | ✓ | | Out |
| Audio tag settings: prefer ARTISTS tag, custom delimiter (default `/|;\`), delimiter whitelist | checkbox/text/textarea | | | | | | ✓ | | Out |

Fetcher Settings dialog (Movies): Fetch Images checkboxes — Primary, Clearart,
Back, Banner, Box, Disc, Logo, Menu, Thumb (`vd`: Primary, Logo, Thumb);
Maximum backdrops per item (1); Minimum backdrop download width (1280). Later.

Notes for Oxevu:

- Jellyfin libraries are typed and may hold several folders. Oxevu's
  "multiple movie/TV roots" maps to one typed library with several roots, or
  one root per library; the per-library knobs above that matter now are type,
  name, roots, enabled, real-time monitoring and series merge.
- The directory browser lists arbitrary server paths (`/`, `/nix/store`,
  `/sys/...`). Oxevu should accept roots only from admin config or a picker
  constrained to allowed base directories, never raw paths in URLs.

### Libraries → Display

| Field | Control | Choices | Now | Class |
|---|---|---|---|---|
| Date added behavior for new content | select | Use date scanned into the library, Use file creation date; metadata value wins | file creation date | **Now** (choose one rule; scanned date is simplest) |
| Display a folder view | checkbox | — | off | Out |
| Display specials within seasons they aired in | checkbox | — | on | Later |
| Group movies into collections | checkbox | — | off | Later |
| Group shows into collections | checkbox | — | off | Later |
| Enable external content in suggestions | checkbox | internet trailers, Live TV | on | Out |

### Libraries → Metadata

| Field | Control | Choices / range | Now | Class |
|---|---|---|---|---|
| Preferred metadata language | select | ISO language list | English | Later |
| Country/Region | select | country list | set | Later |
| Chapter images: Interval | number ≥0 s; 0 disables dummy chapters | — | 0 | Later |
| Chapter images: Resolution | select | Match Source, 2160p … 144p | Match Source | Later |

### Libraries → NFO Settings

Save user watch data to NFO for (user select or None), Save image paths within
NFO, Enable path substitution, Copy extrafanart to extrathumbs. **Out**: these
write into media folders. Reading existing NFO files is a separate, later
read-only feature.

### Related library settings elsewhere

- **General → Paths:** Cache path, Metadata path (blank = default). **Now** as
  runtime data config, outside media roots.
- **General → Performance:** Parallel library scan tasks limit, Parallel image
  encoding limit (blank = based on cores). **Now** as bounded job concurrency.
- **Scheduled Tasks (Library):** Audio Normalization, Download missing lyrics,
  Download missing subtitles, Extract Chapter Images, Generate Trickplay Images,
  Keyframe Extractor, Media Segment Scan, Refresh People, Refresh upcoming and
  missing episodes, Scan Media Library. Maintenance: Clean Activity Log, Clean
  Cache/Log/Transcode Directory, Migrate Trickplay Image Location, Optimize
  database, User data cleanup. **Now:** scan and transcode-scratch cleanup;
  rest Later.

## Users and sharing

User edit page has four tabs. All checkboxes unless noted.

**Profile**

| Field | Class |
|---|---|
| Name (text; letters, numbers, spaces, `- _ ' . @ +`) | **Now** |
| Allow remote connections to this server | Later |
| Allow this user to manage the server (admin) | **Now** (admin role) |
| Allow this user to manage collections | Later |
| Allow this user to edit subtitles | Out |
| Feature access: Live TV access, Live TV recording management | Out |
| Media playback: Allow media playback | **Now** |
| Allow audio/video playback that requires transcoding; allow conversion without re-encoding (remux) | **Now** for video transcode permission; remux/audio Later |
| Force transcoding of remote media sources such as Live TV | Out |
| Internet streaming bitrate limit (Mbps, number, step 0.25) | Later |
| SyncPlay access (select): Create and join groups, Join groups, Disabled | **Now** (watch-party permission) |
| Allow media deletion from: All libraries / per library | Out (media read-only) |
| Remote control: of other users; of shared devices | Later |
| Allow media downloads | Later |
| Disable this user | **Now** |
| Hide this user from login screens | Out (no user picker on login) |
| Failed login tries before lockout (number, ≥-1) | **Now** (fixed default lockout/rate limit) |
| Maximum simultaneous user sessions (number, ≥0) | Later |

**Access:** Enable access to all libraries; when unchecked, a per-library
checklist appears (help: select libraries to share with this user). Enable
access from all devices; when unchecked, a per-device checklist appears.
Per-library access: **Now** (core ACL; must also guard segments). Device
allow-list: Later.

**Parental control:** Maximum allowed parental rating (select of rating
levels), block unrated items per type (Books, Channels, Live TV, Movies, Music,
Trailers, Shows), allow/block items with tags, access schedule. **Later**.

**Password:** current, new, confirm (password inputs). **Now**.

**Add User:** Name, Password, Enable access to all libraries or per-library
checklist. **Now** (same fields).

## SyncPlay (watch parties)

- Server-side exposure is the per-user **SyncPlay access** select above; no
  dashboard page configures SyncPlay globally.
- The header has a **SyncPlay** button. Its menu offers **Join a group** (list
  of existing groups) and **New group** ("Create a new group"). No group was
  created, so the in-group menu and client sync settings were not observed.
- The user Playback settings page has no SyncPlay section in 12.4.

Recommendation: **Now** — permission levels (create/join, join only, none),
create/join/leave group, shared play/pause/seek/queue state, server-authoritative
timeline with drift correction. **Later** — group queue editing, per-client sync
tuning (speed-to-sync vs skip-to-sync), chat. Each member's stream still goes
through normal library ACL and playback-method negotiation.

## Per-user playback preferences (context)

User Settings → Playback (client preferences, not server config): Maximum
allowed audio channels; preferred audio language; play default audio track;
disable VBR audio; home/internet/Cast quality (Auto, 120 Mbps … 420 kbps);
maximum transcoding resolution (Auto, screen, 360p–8K) and "limit maximum
supported resolution"; music internet quality; prefer fMP4-HLS; cinema mode;
play next episode automatically; still-watching prompt; set audio/subtitle
track from previous item; show next video info; Cast version; skip
forward/back length; media segment actions (Intro, Preview, Recap, Commercial,
Outro: None/Ask To Skip/Skip); enable DTS, TrueHD; H.264 High 10; shorter HLS
segments; preferred transcode video codec (Auto, H264, HEVC, AV1) and audio
codec (Auto, AAC, AC3, ALAC, DTS, FLAC, Opus); audio normalization; always
remux FLAC/MP3.

Relevant **Now**: quality cap (Auto + a few bitrates), fMP4 HLS, auto-play next
episode, track memory. Codec preferences and segment skipping: Later.

## Proposed next-phase scope

Implement now:

1. FFmpeg settings: binary path (config), acceleration `none | nvenc`, NVDEC
   codec allow-list with 10-bit toggle, HW encode on/off, tone mapping on/off
   with BT.2390 defaults, subtitle extraction, throttling and segment deletion
   with fixed bounds, muxing queue size, scratch dir under runtime data. CPU
   fallback when NVENC fails.
2. Resume thresholds: min/max percentage and minimum duration.
3. Libraries: typed (movies/shows) with multiple roots each, enable flag,
   real-time monitoring toggle with rescan fallback, series merge across roots,
   date-added rule. Roots from constrained admin input only.
4. Users: admin flag, disabled flag, password, playback and video-transcode
   permissions, per-library access (all or list), login lockout, SyncPlay
   access level.
5. Watch parties: groups with create/join permission and a shared timeline.
6. Bounded scan and image job concurrency.

Later: other HW backends (VAAPI/QSV), HEVC/AV1 output, encoder preset and CRF,
tone-mapping tuning, audio downmix options, remote bitrate limits, trickplay,
chapter images, metadata providers and refresh schedules, collections,
parental controls, device allow-lists, session limits, segment skipping.

Out of scope: NFO/artwork/subtitle/lyrics writing into media folders, media
deletion permissions, Live TV, subtitle downloading, music/book-specific
settings, folder view, external content suggestions.

## Uncertainties

- Version string reads "12.4"; this inventory reflects that build only.
- The HW-type switch was not saved, so fields that appear only after a save or
  a server capability check may be missing.
- SyncPlay in-group controls and client sync settings were not inspected
  because that needs creating a group.
- Metadata Manager, Networking, Devices, Plugins, Live TV and Backups were not
  inventoried.
- Movies and Shows were inspected through Manage library with live values;
  other types only through the Add form with defaults.
