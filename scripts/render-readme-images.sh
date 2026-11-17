#!/usr/bin/env bash
set -euo pipefail
repo_root=$(cd "$(dirname "$0")/.." && pwd)
browser=${BROWSER_BIN:-google-chrome-stable}
scratch=$(mktemp -d -t oxevu-readme-render.XXXXXX)
trap 'find "$scratch" -depth -delete' EXIT
for screen in hero discover seasons mobile; do
  "$browser" --headless=new --no-first-run --disable-extensions --hide-scrollbars \
    --user-data-dir="$scratch/profile" --force-device-scale-factor=2 \
    --window-size=1600,1000 --virtual-time-budget=2000 \
    --screenshot="$scratch/$screen.png" \
    "file://$repo_root/assets/screenshots/source/$screen.html" >"$scratch/chrome.log" 2>&1
  ffmpeg -hide_banner -loglevel error -y -i "$scratch/$screen.png" \
    -frames:v 1 -c:v libwebp -quality 82 -compression_level 6 \
    "$repo_root/assets/screenshots/$screen.webp"
done
