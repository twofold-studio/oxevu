# Integrations

Settings → Media → Metadata (TMDB, OMDb, studio images) and Subtitles
(OpenSubtitles) are administrator-only. Fields save when editing ends;
blank secret fields preserve existing credentials. Remove credentials clears them.
Credentials live in `integrations.json` under the server data directory with mode
0600. API responses contain configuration flags, never passwords or API keys.

- **Studio images:** enabled by default, no personal key. Public images come from
  [Jellyfin Artwork](https://github.com/jellyfin/jellyfin-artwork), licensed under
  [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). This uses the
  public artwork source, independently of any Jellyfin installation.
- **TMDB:** configure an application API key or read access token, then enable it.
  [Noncommercial use is free with attribution](https://developer.themoviedb.org/docs/faq).
- **OMDb:** configure an [API key](https://www.omdbapi.com/apikey.aspx), then enable
  it. The separate high-resolution poster service requires patron access.
- **OpenSubtitles.com:** configure an application API key, username and password,
  then enable it. [Its API requires the application key](https://opensubtitles.tawk.help/article/getting-started)
  even when account credentials are available. Downloads count against provider quotas.

Identify on a title opens provider search. Review the candidate and apply the
match; ambiguous filenames should not silently select a different film. Series
matching applies episode information using existing season and episode numbers.
Metadata survives rescans. Cached artwork and downloaded subtitles stay in
Oxevu's data directory; source media remains unchanged.

Local artwork and subtitles work without online credentials. Embedded bitmap
subtitles require rendering into the video and are not supported yet. Text
subtitles are converted to WebVTT for browser playback.

This product uses the TMDB API but is not endorsed or certified by TMDB.

The TMDB mark in settings is an unmodified approved logo downloaded from its
[official attribution page](https://www.themoviedb.org/about/logos-attribution).

## Registration notes

Application keys can belong to the server operator; each viewer does not need
their own key. Register Oxevu with each provider rather than reusing Jellyfin's
application credentials. Keep keys in the server data directory and a password
manager, outside Git. Registration may require email verification and contact
information; a TMDB account alone does not issue an application key.

Provider redirects are limited to three hops on the original scheme, host and
port. This supports OpenSubtitles search normalization while preventing
application credentials from following a redirect to another origin.
