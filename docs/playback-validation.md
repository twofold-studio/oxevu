# Playback phase validation

Observed on vd, 2026-11-15. This records checks completed so far, not full acceptance.

## Backend

- Backend suite: 37 passing tests reported by implementation agent, including
  actual CPU/NVENC output, offset/duration handling, GPU failure fallback,
  authorization, concurrency limits, HDR rejection and active-child kill/reap.
- Formatting and strict workspace Clippy passed in the Nix shell.
- Parent copied the live SQLite database using SQLite backup into a separate
  runtime directory and ran the new backend on loopback port 3016.
- Migration preserved the existing administrator, two libraries and sample catalog.
- Actual runtime probes reported FFmpeg and GPU0 NVENC available, hardware decode
  unavailable. A real Anna conversion starting at 20 seconds reached ready with
  method `nvenc`, and its authenticated HLS manifest was served successfully.
- Parent created a party, sought to 30 seconds, checked its revision and left.
  The test stopped its playback session and closed its party afterward.

## Frontend integration

- Implementation agent reports 89 frontend tests passing, type-check/lint/format
  checks and build passing, plus Svelte analysis of changed components.
- On a generated legal fixture library: direct MP4, NVIDIA AVI conversion,
  forced conversion, rapid seek coalescing, absolute resume and scratch cleanup.
- Two browser users: host pause/play/seek followed within approximately 0.2s
  in the local fixture test, title changes and room close reached followers.
- On the migrated real-library copy at port 3016: Arcane S1E1 HEVC/EAC3 played
  through NVIDIA at 1920x816; seeking to 1200s and back to 1180s recreated the
  session correctly. Scratch was empty after leaving. Test progress was written
  only to the copied database.
- Settings save/persist, two-root library creation/scanning and a user without
  library access were exercised in the browser. Grant changes revoked sessions.

GPU1, physical/mobile devices and synchronization over a real remote network
remain untested. Browser checks above were performed by the implementation agent.

## Current limits

- H264/AAC output; no hardware decoding or HDR tone mapping. Unsupported settings
  must be rejected or visibly unavailable, not saved as working controls.
- Scratch limits are sampled every 100 ms and can briefly overshoot.
- Parties are memory-only and close on restart. Limits are 32 rooms, 16 members,
  90-second member leases and five-minute room leases.
- No trickplay, subtitle selection/burn-in, automatic file watching or online
  metadata recognition in this phase.
