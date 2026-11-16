# Seeking and conversion cache

The browser buffers toward 90 seconds ahead, subject to a 128 MB byte budget
and a 120-second upper duration bound. It retains 60 seconds behind. Seeking
inside the current produced playlist reuses that session, including segments
not yet downloaded to the browser. Native HLS buffering remains browser-managed.

Converted chunks persist outside media folders in `transcodes/oxevu-cache`
under the data directory, or beside active scratch under `OXEVU_TRANSCODE_DIR`.
The default cache limit is 5 GB; administrators can change it or disable reuse
in Settings → Playback → Scratch and cache. Old entries are evicted by last access. Active sessions
retain their own hardlinks, bounded separately by the scratch limit.

Only finalized segments are cached. Source identity, FFmpeg identity, pipeline
version and effective conversion settings determine compatibility. A changed
file or quality plan misses the cache. Authentication, library grants and
conversion permissions still apply to every playback request.

The cache can restore a completed video or a partial range with at least 30
seconds remaining. A partial playlist has a definite end; the player starts a
new session there if the title continues. This boundary can still require a
short startup wait. Cache reuse does not make every seek instantaneous.

Auto starts at the server's quality limits unless Data Saver is enabled. Coarse
browser downlink hints do not lower quality. Sustained segment transfer samples
can reduce quality; time waiting for the first byte is excluded. The menu shows
the current preset, bitrate ceiling and actual decoded pixel dimensions.

NVIDIA encoding requests IDR frames for forced keyframes so the intended
two-second HLS segments are actually produced. Source probe results use a
bounded memory cache invalidated by file identity and timestamps.

## Validation on vd (2026-11-15)

Chrome playback of Arcane on the disposable integration instance reused the
same segment inode from the cache without starting an encoder. A backward seek
into that range took 444 ms from the seek action to a rendered frame. Playback
continued into a new session at the partial cache boundary. This is one local
measurement, not a latency guarantee.

The browser showed Auto 1080p / 8 Mbps with decoded dimensions 1920×816 and about
99 seconds buffered ahead. Screenshots checked the selected library type, folder
suggestions and filtered subtitle menu. Keyboard selection completed a folder path.
All four explicit GPU tests passed, including real HEVC10 Arcane output and
two-second segment assertions. An initial run omitted the required fixture path;
the rerun supplied it. Experimental low-latency NVENC flags caused startup failure
and were removed before acceptance.
