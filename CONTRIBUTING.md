# Contributing

See [development](docs/development.md) for the Nix environment, shared API
contracts and local server, and [architecture](docs/architecture.md) for the
crate boundaries. Read [AGENTS.md](AGENTS.md) before changing code. Mobile build
and device acceptance steps are in [mobile](docs/mobile.md).

## README images

The cover and gallery are composed from real production screenshots. Their HTML
layouts and shared styling live in `assets/screenshots/source/`. Raw PNG captures
belong in the ignored `source/shots/` directory; never commit session cookies,
credentials, account details or browser profiles.

Capture the running production service at 2× scale: Movies, Arcane details,
Arcane scrolled with its Seasons heading 110 pixels below the viewport top,
and Discover at 1280 × 800 CSS pixels. Save
them as `movies.png`, `series.png`, `seasons.png` and `discover.png`. At 390 × 844,
capture Movies, Arcane details and Shows as `mobile-movies.png`,
`mobile-series.png` and `mobile-shows.png`. Wait for visible artwork to load.
These are responsive web captures, not evidence of physical-device playback.

With Chrome and FFmpeg available, render the four 3200 × 2000 WebP images:

```sh
bash scripts/render-readme-images.sh
```

Set `BROWSER_BIN` if Chrome has a different executable name. Review every image
at README width and full resolution. Keep the HTML layouts alongside the output
so future captures can be regenerated without rebuilding the composition.
