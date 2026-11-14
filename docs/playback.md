# First playback slice

1. Start the server with persistent application data. Bootstrap an administrator through a one-time local setup mechanism; no default credentials.
2. Sign in, configure a read-only movie root and start a scan.
3. Probe a small test library, list items and open an item details view.
4. Create an authorized playback session. Select direct delivery when the client reports compatible container, codecs and tracks. Explain the selected playback method.
5. Serve byte-range requests correctly for direct playback. Verify seeking near the beginning and end, cancellation and concurrent sessions.
6. Persist per-user position, duration and completion with periodic updates and a final best-effort update. Resume after browser/server restart.
7. Add FFmpeg HLS conversion for unsupported media, including session cancellation, seeking, bounded scratch storage and cleanup. Prefer remuxing when only the container is incompatible.
8. Add audio/subtitle selection. Define supported formats explicitly and report unsupported tracks. Treat subtitle burn-in as a transcode requirement.
9. Add NVIDIA acceleration with an explicit CPU fallback. Report actual encoder/decoder selection and failure reasons. Avoid retry loops on broken GPU configuration.

Server-side capability negotiation must validate requested tracks and paths. Session or segment credentials must be short-lived and excluded from logs; reject unauthorized direct, manifest and segment requests. Never trust client-supplied command-line arguments.

Acceptance on vd: sign-in; read-only movie and TV scans; direct play; range seeking; persisted resume; unsupported-video conversion; NVIDIA conversion; CPU fallback; independent user progress; child-process and scratch cleanup. Include ffprobe input properties, browser/device, selected method and measured CPU/memory in the test record. Mobile acceptance follows the packaging spike in architecture.md.
