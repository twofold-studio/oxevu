Run with `cargo run -p oxevu-server` inside `nix develop`.
Defaults: 127.0.0.1:3000, Origin http://localhost:3000, data
$HOME/.local/share/oxevu. Override OXEVU_BIND, OXEVU_ORIGIN and
OXEVU_DATA_DIR. Use an absolute runtime directory outside source/media roots.
ffprobe must be on PATH.

On the server machine read DATA_DIR/setup-token (mode 0600). Submit
POST /api/v1/auth/setup with Origin matching OXEVU_ORIGIN,
X-Oxevu-Setup-Token containing that token, and JSON username/password.
Passwords require 12 bytes minimum. Token rotates at startup; setup only
works before the first user exists. Keep bind on loopback during setup.
Use HTTPS and set OXEVU_ORIGIN accordingly for remote access.
All registered users are administrators in this phase; there is no user creation API.
Sessions expire after seven days; logout revokes all sessions for that user.

Frontend dist is embedded at build time when present; rebuild after frontend
changes. Missing dist produces a useful HTTP 503 page.

Scanning is serial, with a 30-second probe timeout and 100000-entry cap.
Symlinks are skipped. TV names require SxxExx. Rescans preserve IDs/progress;
missing files are retained. No conversion, artwork, or metadata providers.
