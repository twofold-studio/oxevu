# Acquisition acceptance, 2026-11-16

Checked on vd with disposable application data. Existing services and media
were not used as download/import destinations.

| Check | Result |
| --- | --- |
| Movie request → release → real loopback transfer → import → authenticated range playback | Passed with generated H264 media; source retained and bytes matched |
| Two users, overlapping TV scopes and a shared pack | Three wanted episodes imported once; the fourth stayed unavailable |
| RSS and missing searches | Deduplicated; offline destination suspended searches |
| Competing episode releases | One reservation per unit; seeding episode 1 did not block episode 2 |
| Restart | Fresh acquisition runtime rechecked files; an actual server restart resumed the fixture seed and retained requests/imports |
| Import recovery | Published journal replay scanned without another row or copy |
| Filesystem safety | Different existing content, changed sources and symlink escapes rejected |
| Existing-data adoption | Separate inode copies, checksums matched, paused until explicit recheck/resume |
| Permissions | HTTP authentication, admin boundaries, Origin, request ownership/quotas and library ACLs checked |
| Migration | Preview token bound mappings; owner retained; repeat import did not duplicate requests |

Browser checks covered real TMDB search/artwork, request creation, destination
selection/autosave, indexer creation and capability test, release search/profile
results, paused torrent upload, reviewed adoption, seeding, library import and
playback. The imported 12-second clip reached `readyState=4`; its clock advanced
and right-arrow seeking moved by five seconds. Original and imported SHA-256
digests matched. The request became fulfilled and survived a server restart.

Desktop and 390px screenshots were inspected. Mobile Activity stayed 390px wide,
and its More menu exposed secondary destinations. Task modals stayed centered,
one adoption modal was open at a time, and closing returned focus to the files
button. No browser errors were reported. A browser wait targeting an icon's
non-visible label timed out; the subsequent accessible-button check passed.

The default Rust suite passed 61 tests, with four hardware-specific tests ignored.
The two explicit CUDA tests and NVENC/failure-fallback test also passed on GPU0,
including a real HEVC10 Arcane excerpt. The startup benchmark was not rerun.
Web type/lint/format/build checks and all 114 web tests passed. Svelte tooling
reported no component issues; generic autosave-effect suggestions were reviewed.

An additional public-network smoke test used the
[official Ubuntu 24.04.5 server torrent](https://releases.ubuntu.com/releases/24.04/ubuntu-24.04.5-live-server-amd64.iso.torrent).
With direct routing, DHT enabled, a 256 KiB/s download cap and 64 KiB/s upload cap,
the application verified 18,827,264 bytes and reported 92 live peers. Pause closed
the peer TCP connections. After a server restart, the job stayed paused until
explicit resume, rechecked its files and advanced to 24,856,576 verified bytes.
The resumed snapshot reported 51 TCP and four uTP peers.

During the transfer, a generated video reached `ready`, `method=nvenc`,
`decoder=cuda`; its HLS segment returned HTTP 200 with 857,092 bytes. This was a
partial image transfer, not a full ISO checksum or sustained load test.

This proves the native workflows with legal fixtures and a short public transfer.
Broader tracker compatibility, sustained transfer/hash/conversion contention, IPv6 behavior,
private tracker rules and real-device checks remain production acceptance work.
The prototype harness has separate TCP/v1 evidence in [torrent-engine.md](torrent-engine.md).
Downloads and imports remain opt-in. Existing applications are not retired.

## Release search follow-up · 2026-11-16

The regression search returned 115 distinct release IDs and 115 cache rows,
with identical results on a repeat. XML tests cover split text/entity fragments,
numeric references, CDATA, invalid entities, accumulated field limits and full
100-item pages. Cross-source infohash matches retain an additional-source list.

A generated legal torrent followed HTTP 301 → magnet, discovered metadata from
a loopback peer and reached seeding. The job retained its release label, source,
size and cached magnet. Legacy jobs can read labels from their retained release
row. Redirect tests reject external targets and loops, accept one same-origin
HTTP redirect, identify HTTP 429, and stop a deliberately stalled request after
30.01 seconds. No public release was downloaded for this review.

Browser checks used disposable accounts/data and four loopback sources at
1280×720 and 390px wide. Observed: 115 unique torrents from 116 hits, one named
rate-limit failure, additional sources, cancellation restoring the prior result,
separate empty/all-failed screens, and no duplicate-key error. Changing profile
disabled old results. A 30 GiB release displayed the temporary profile's 10 GiB
limit and required confirmation. An existing failed job showed its error rather
than a queue-success message. Activity distinguished two releases of one movie
and stated that completed files were not imported because imports were off.

The temporary profile saved zero as unlimited through the UI; the default stayed
at 20480 MiB. Browser views had no sideways overflow. Unit checks covered search
cancellation, timeout, duplicate IDs, queue errors and ETA. Web checks passed all
143 tests plus type, lint, format and build. The full Rust suite passed 69 tests
with four existing hardware checks skipped; strict Clippy and format checks passed.
The extended timeout and legacy-label regressions were also run separately.

Existing cached releases and job references are preserved; there is no one-off
cache-deletion migration. Torrent workers, seeding and import publication rules
remain unchanged. Real-provider availability and long-running transfers still
depend on those services; the browser review used local fixtures.

After deployment, the original live movie search returned 115 source hits from
four successful indexers in 3.9 seconds. Grouping matching infohashes produced
65 torrents with 65 unique IDs; 46 rows listed additional sources. This checked
search only: no live download was started and no profile setting was changed.
