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
