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

- H264/AAC output; no HDR tone mapping. Unsupported settings
  must be rejected or visibly unavailable, not saved as working controls.
- Scratch limits are sampled every 100 ms and can briefly overshoot.
- Parties are memory-only and close on restart. Limits are 32 rooms, 16 members,
  90-second member leases and five-minute room leases.
- No trickplay, subtitle selection/burn-in, automatic file watching or online
  metadata recognition in this phase.

## Seek and hardware-decode follow-up

- Frontend regression tests cover stop acknowledgement before replacement,
  cancellation of starting sessions, latest-target coalescing and stop failures.
- Hardware tests on GPU0 passed CUDA decode/scaling for generated HEVC10bit and
  a real Arcane excerpt: 471 output frames at 720p, CUDA decoder and NVENC encoder.
  NVIDIA dmon observed 36% decoder and 100% encoder utilization during the test.
- Invalid GPU fallback on produced playable CPU output; fallback off failed and
  cleaned scratch. Hardware decoding plus CPU encoding also passed.
- HLS now uses two-second EVENT segments with an explicit start offset. Single-run
  FFmpeg first-manifest measurements improved CPU startup; NVENC improvement was
  small. These timings alone do not prove browser seek latency improvements.
- Hardware probe covers H264 CUDA; HEVC10bit is covered by hardware tests. Other
  codecs/profiles and GPU1 remain unverified.

## Quality, subtitles and integrations (2026-11-15)

- Web checks: type-check, lint, format, 99 tests and production build passed.
- Server checks: 41 tests, strict Clippy and format passed, including the
  scratch-accounting regression fix.
- All three explicit GPU acceptance tests passed with the real Arcane fixture:
  CUDA decoding/scaling, NVENC and CPU output, GPU failure policy and cleanup.
- Browser against the integration database on 3016: integration defaults rendered,
  studio toggle persisted automatically, 720p/4 Mbps was applied to a CUDA/NVENC
  session, and a rapid seek burst ended at 900 seconds without a session-limit
  response while the server limit was one. Every replacement awaited stop (204)
  before create (200). English text subtitles loaded with 306 offset-adjusted cues.
- The keyless studio endpoint returned a real Netflix SVG from the attributed
  public artwork source. A temporary integration-copy metadata record was restored.
- Long playback exposed an existing scratch sampler race: an FFmpeg temporary
  file could disappear between enumeration and stat, incorrectly tripping the disk
  limit. Fixing the sampler preserves errors for unreadable/missing scratch roots.
- After the sampler fix, a fresh real Arcane CUDA/NVENC 720p/4 Mbps session
  stayed ready through 65 seconds of polling and was explicitly stopped.
- No live TMDB, OMDb or OpenSubtitles credentials were configured for testing.
  Their live search/download acceptance remains outstanding. Recognition currently
  requires an explicit reviewed match; scanning does not make automatic network matches.
- Native HLS uses browser network hints/server defaults; sustained throughput
  adaptation is available through hls.js. Bitmap subtitle burn-in is unavailable.
