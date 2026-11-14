<p align="center">
  <img src="assets/logo.png" alt="Oxevu's orange loop and play symbol" width="160" />
</p>

<p align="center">
  <a href="GOAL.md"><img src="https://img.shields.io/badge/project-plan-e87935" alt="Read the project plan" /></a>
</p>

<h1 align="center">Oxevu</h1>

<p align="center">Your media, in one place.</p>

Oxevu is a self-hosted media app in early development. It aims to bring a home server's player, library, requests, indexers, and downloads into one Rust executable, with a shared Svelte interface for the web, Android, and iOS.

The project will replace an existing Jellyfin, Sonarr, Radarr, Prowlarr, qBittorrent, and Seerr setup one module at a time. Each module should offer the smallest set of features that makes it useful at home.

Planned capabilities:

- **Watch** movies and shows, with subtitles, saved progress, and hardware-accelerated transcoding.
- **Find** and request media through a shared catalog and configured torrent indexers.
- **Manage** downloads and import completed movies and episodes into the library.

## Start here

Read [the project goal](GOAL.md) for the modules, their first useful milestones, and the order of work. The first local server and web client now support sign-in, read-only scans, catalog browsing, original-file playback, and saved progress. See [development setup](docs/development.md#run-the-first-application) to run it. Transcoding and mobile packaging are still pending.

The first milestone is to scan an existing movie or TV folder and play a video in a browser, including seeking, resume, and an NVIDIA transcode on the home server. Oxevu will run alongside Jellyfin while this is developed and tested.
