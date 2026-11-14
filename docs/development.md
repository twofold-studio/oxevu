# Development environment

Run `direnv allow` from the project root, or enter with `nix develop`. The first activation downloads the SDK and emulator images and can be large. The flake explicitly enables Android SDK license acceptance and unfree packages; review the Android SDK terms before entering it.

The shell provides Rust/Cargo/rustfmt/Clippy/rust-analyzer, Node.js/pnpm, pkg-config/OpenSSL, FFmpeg/ffprobe, SQLite, Java 17, Android API 35/build tools, adb, avdmanager and an Android emulator with a Google APIs system image matching the host architecture. Versions are pinned by flake.lock. Mobile framework-specific NDK or native bridge dependencies will be added when that framework is chosen.

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
S02E01 respectively. Add them as Movies and TV libraries, then scan. They are
separate copies; originals remain in Jellyfin's Blockade folders.

For frontend development run the backend with
`OXEVU_ORIGIN=http://localhost:5173`, then `pnpm dev` in apps/web. The proxy
forwards `/api` to port 3000. See crates/server/README.md for server configuration.

Current limits: original-file delivery only, no conversion, track selection or
online metadata recognition. Local artwork sidecars and generated thumbnails are supported. Browser support depends on container/codecs; copied
MKV samples may require the later remux/transcode phase. Rescans retain missing
files, probe output is not yet size-capped, and runtime/media directory separation
is an operator requirement. NVIDIA and physical mobile testing remain outstanding.

Artwork caches currently have no eviction or source-change invalidation. Rescan
after adding local sidecars. Recognition against an external metadata catalog is
not implemented yet; copied sample artwork is optional local input.
