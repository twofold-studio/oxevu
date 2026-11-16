# Development environment

Run `direnv allow` from the project root, or enter with `nix develop`. The first activation downloads the SDK and emulator images and can be large. The flake explicitly enables Android SDK license acceptance and unfree packages; review the Android SDK terms before entering it.

The shell provides Rust/Cargo/rustfmt/Clippy/rust-analyzer, Node.js/pnpm, pkg-config/OpenSSL, FFmpeg/ffprobe, SQLite, Java 17, Android API 35/build tools, adb, avdmanager and an Android emulator with a Google APIs system image matching the host architecture. Versions are pinned by flake.lock. Mobile framework-specific NDK or native bridge dependencies will be added when that framework is chosen.

The optional [torrent engine harness](torrent-engine.md) uses CMake, also provided
by the shell, to build its native TLS dependency. It does not enable downloads in
the application.

## Android

Create a user-local virtual device once:

```sh
avdmanager list device
avdmanager create avd --name oxevu --package "system-images;android-35;google_apis;x86_64"
emulator -avd oxevu
adb devices
```

On ARM hosts replace `x86_64` with `arm64-v8a`. Emulator state is stored in your home directory, outside the Nix store. Use `10.0.2.2` to reach the host from the standard Android emulator.

Linux hardware acceleration requires accessible `/dev/kvm` and host virtualization support. A project shell cannot grant KVM permissions or install GPU drivers. Verify those on the host before launching an emulator. NVIDIA transcode validation also needs the host driver and a compatible FFmpeg build; inclusion of FFmpeg alone does not establish NVENC support.

## iOS

Apple's Simulator runs only on macOS with Xcode. It cannot be installed into this Linux shell. On a Mac, install full Xcode and its iOS Simulator runtime, complete first-run setup, then use:

```sh
xcode-select -p
xcrun simctl list devices available
open -a Simulator
```

The Apple silicon macOS Nix shell supplies shared tooling; Xcode and simulator runtimes remain Apple-managed host dependencies. Signing and real-device deployment need separate Xcode configuration. Do not claim iOS support based on browser emulation alone.

References: [Nixpkgs Android tooling](https://nixos.org/manual/nixpkgs/unstable/#android), [Apple Simulator setup](https://developer.apple.com/documentation/safari-developer-tools/installing-xcode-and-simulators).

## Changelog workflow

Write meaningful changes directly under `Unreleased` in CHANGELOG.md, using
Added, Changed, Fixed, Removed or Decisions sections as needed. No separate
fragment files or automatic commit-to-prose generation are required.

- `just changelog-check` checks version/history consistency.
- `just release 0.1.1` moves pending notes into a dated release and updates VERSION.

The helper uses Python's standard library; `python3 scripts/changelog.py check`
and `python3 scripts/changelog.py release 0.1.1` also work without just. Release
preparation requires pending bullet entries and a greater version. It does not
commit, tag, publish, or update future package manifests. Review the diff and run
project checks before committing a release.

## Run the first application

Build the web app before compiling the server to embed the interface:

```sh
cd apps/web
pnpm install
pnpm build
cd ../..
OXEVU_DATA_DIR="$HOME/.local/share/oxevu-dev/server" cargo run -p oxevu-server
```

Open `http://localhost:3000`. First-run setup asks for the token in the data
directory's `setup-token` file and a password of at least 12 bytes. Keep the bind
on loopback during setup. Never commit the token or runtime database.

Sample library roots on vd are `/home/fractal-tess/.local/share/oxevu-dev/media/movies`
and `/home/fractal-tess/.local/share/oxevu-dev/media/tv`, containing Anna and Arcane
both seasons (18 episodes) respectively. Add them as Movies and TV libraries, then scan. They are
separate copies; originals remain in Jellyfin's Blockade folders.

For frontend development run the backend with
`OXEVU_ORIGIN=http://localhost:5173`, then `pnpm dev` in apps/web. The proxy
forwards `/api` to port 3000. See crates/server/README.md for server configuration.

Settings open from the account menu at `#/settings/<group>/<page>`, for example
`#/settings/media/libraries` or `#/settings/playback/resume`. The page catalog,
legacy aliases (`#/settings/playback`, `#/settings/acquisition`, …) and role
rules live in `apps/web/src/lib/state/settings-pages.ts`. Only administrators
manage server settings, libraries and users; delegated indexer managers get
Sources → Indexers. Existing accounts migrate to administrators.
Create individual non-admin users with explicit library/playback permissions.

New settings belong on a focused page with at most three related control groups.
Register its route, label and role in `settings-pages.ts`; use the shared settings
frame and rows. Keep navigation in the settings sidebar (phone: settings index),
not nested tabs. For whole-record APIs, use `SettingsDocument` and declare only
the fields the page owns so edits on another page survive autosave.

The player selects original, remux or H264/AAC HLS delivery. Automatic conversion
uses NVIDIA encoding when configured, with an optional CPU fallback. Settings
include GPU index, height/bitrate/CRF, concurrency, scratch limits and resume rules.
NVIDIA hardware decoding is available when its runtime probe succeeds. Text subtitle selection is supported; image subtitle burn-in, HDR conversion and trickplay are unsupported.
Optional provider setup is described in [integrations](integrations.md).
FFmpeg path (`OXEVU_FFMPEG`) and scratch directory (`OXEVU_TRANSCODE_DIR`) are server
configuration; scratch must not overlap media roots or the source tree.

Watch parties are under `#/parties` or Watch together on an item. The host controls
play/pause/seeking and title changes; members need their own library permissions.
Parties are in memory and disappear on server restart. Local synchronization tests
do not establish remote-network or mobile-device behavior.

Rescans retain offline roots safely. File watching and automatic metadata matching
during ordinary scans are deferred. Manual provider identification and metadata
attached by acquisition imports are available. See [validation](playback-validation.md).

Artwork caches currently have no eviction or source-change invalidation. Rescan
after adding local sidecars. Provider artwork is available after identification;
local sidecars take precedence.

## Shared interface motion

Use the tokens in `apps/web/src/app.css` and `lib/motion.ts` for consistent timing.
`animate-rise`, `animate-drop` and `animate-fade` handle entrances; `press` and
`lift` add restrained interaction feedback. Artwork handles image reveal itself.

Use `SwapView` for changing content without collapsing its region:

```svelte
<SwapView key={selectedSeason}>
  {#snippet children(season)}
    <EpisodeList episodes={episodesFor(season)} />
  {/snippet}
</SwapView>
```

Render from the snippet's key so outgoing content stays correct. Reduced motion
skips travel and view transitions. Keep selection visible without animation.
Season URLs accept `?season=N`; returning from an episode restores that season.

Settings edits save automatically after a short pause. Invalid values are not sent.
Creating users/libraries, changing passwords and removing libraries remain explicit
actions. Enable Hardware decoding under Settings → Playback → Conversion to use CUDA decoding
for new converted streams; the player delivery menu reports the actual decoder.

Acquisition is under `#/discover`, `#/requests`, `#/wanted`, `#/activity` and
the Downloads and Sources settings groups. See [setup, migration and rollback](acquisition.md).
Keep downloads/imports disabled until storage and title ownership are configured.
The acquisition test suite uses disposable generated media and loopback peers;
it does not use the household's indexers or download real releases.
