# Mobile clients

Android uses Capacitor 8 to package `apps/web/dist`. The Svelte screens, generated
REST client, routing and player are shared with the web build. The native project
lives in `apps/mobile/android`; `apps/web/capacitor.config.ts` reserves an iOS path
for later packaging on macOS. No remote HTML is loaded into the native bridge.

## Build Android

From the repository root:

```sh
nix develop
cd apps/web
pnpm install --frozen-lockfile
pnpm mobile:build
```

The development APK is at
`apps/mobile/android/app/build/outputs/apk/debug/app-debug.apk`. Install with
`adb install -r <apk>`. This is debug signed, not a Play Store release. Release
signing keys belong outside the repository. VERSION supplies the app version name.

The first screen asks for an HTTP or HTTPS server origin. Use a LAN address or
NetBird hostname reachable from the phone; `localhost` refers to the phone.
Change server from the account menu or connection/sign-in screen. Server addresses
are saved in private Android preferences; passwords and session tokens are not
stored in JavaScript preferences. Cleartext HTTP is permitted for existing LAN
and mesh servers. Prefer HTTPS outside a trusted network; TLS errors are never
bypassed. No server exposure, firewall or production port changes are required.

## Authentication and media

The server keeps its existing HTTP-only, SameSite=Strict sessions. Generated JSON
API calls use CapacitorHttp and its native cookie jar. The request's Origin is the
explicitly selected server; redirects are disabled. Aborted requests discard their
result, but CapacitorHttp does not expose cancellation of an in-flight native call.

On Android, a small `BridgeWebViewClient` forwards read-only `/api/v1/` resources
from the bundled UI origin to the selected server. This lets images, subtitle
fetches, native video and HLS requests carry the same native session without
putting tokens in URLs or weakening browser cookie rules. It streams response
bodies, forwards Range/If-Range, limits concurrent readers, rejects redirects and
document navigations, and disconnects requests when streams close. It does not
buffer whole movies or cache authenticated media across server changes.

The server's web client remains same-origin and uses ordinary browser fetch.
The Android container supplies fullscreen support; the shared App plugin handles
Back navigation. Background playback, downloads, casting and picture-in-picture
are not implemented. Native player integration remains a separate decision if
WebView playback fails the device checks.

## Acceptance

On 2026-11-17, the API 36 x86_64 emulator connected to a disposable server,
signed in, browsed a legal generated H.264/AAC fixture, retained sign-in after
an app restart, and played original and NVIDIA-converted HLS video. The media
bridge returned a verified 206 byte range; the shared forward-seek control
continued HLS playback. Fullscreen entered successfully and Android Back exited
fullscreen without leaving the player. Resume was checked with the disposable
server's minimum-duration rule reduced for its short test clip. The real server's
settings and media were not changed.

Build/lint and emulator evidence do not establish physical-device compatibility.
Before release, test sign-in persistence, direct play, HLS conversion, seeks,
resume, subtitle/audio selection, rotation, fullscreen/Back, network interruption,
and logout/server changes on a physical Android device. Check that other accounts
and servers cannot reuse cached artwork/media. iOS needs equivalent native media
transport plus macOS/Xcode build and device validation; the shared UI requires no
rewrite, but the Android bridge is not an iOS implementation.
