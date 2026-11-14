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

## Pending integration

Browser playback, seeking and resume with the new frontend; independent browser
participants synchronized across play/pause/seek; reconnect/autoplay behavior;
settings and user administration through the UI. GPU1 and physical mobile devices
are untested. Do not claim these from API tests or generated fixtures alone.

## Current limits

- H264/AAC output; no hardware decoding or HDR tone mapping. Unsupported settings
  must be rejected or visibly unavailable, not saved as working controls.
- Scratch limits are sampled every 100 ms and can briefly overshoot.
- Parties are memory-only and close on restart. Limits are 32 rooms, 16 members,
  90-second member leases and five-minute room leases.
- No trickplay, subtitle selection/burn-in, automatic file watching or online
  metadata recognition in this phase.
