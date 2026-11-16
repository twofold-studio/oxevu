# Proposed subtitle alignment contract (not implemented)

Deferred at the owner's thread handoff. None of the routes or Auto-align UI below
is implemented or deployed. This proposal records the intended contract for the
next agent; reconcile it with the real implementation before building clients.

Chorus's current English WAV2VEC2 forced aligner is unsuitable for matching
translated English subtitles to Japanese dialogue. The proposed first trial uses
[FFsubsync's speech-activity alignment](https://ffsubsync.readthedocs.io/en/latest/reference_types.html),
retaining original files and evaluating a separate output copy. No aligner has
been installed, and no actual movie/subtitle alignment has been run in this work.

- `GET /api/v1/items/{item}/subtitles/alignment` returns
  `{available: boolean, reason: string | null}`. Checks item playback access and
  reports unavailable for incomplete/absent local media or missing sync tooling.
- `POST /api/v1/items/{item}/subtitles/{track}/align` returns HTTP 202 `{job}`.
  Requires playback and conversion permission, like subtitle download. Only a
  supported track already belonging to this item is accepted; no client paths.
- `GET /api/v1/items/{item}/subtitles/alignment/{job}` returns `{job}` with the
  same item and requesting-user authorization. A missing/restarted job is 404.

```ts
interface SubtitleAlignmentJob {
  id: string;
  status: "queued" | "running" | "succeeded" | "failed";
  step: "preparing" | "analyzing" | "publishing" | "done";
  error: string | null;
  result: {
    track: SubtitleTrack; // existing track shape
    offset_seconds: number;
    method: "speech_activity";
  } | null;
}
```

The first implementation estimates a bounded constant offset using audio speech
activity and FFsubsync. It does not claim word-level forced alignment, translation
matching or correction of different scene cuts. It keeps the original subtitle
and publishes a separate selectable aligned copy. At most one analysis runs at a
time, with bounded retained job records and process time/output limits. Nothing
aligns automatically on page/menu opening or every download in this first trial.
The owner explicitly chooses Auto-align; the calculation itself is automatic.

UI: check availability once when the subtitle menu opens. A selected supported
track can open a centered Auto-align dialog within the fullscreen player portal.
Start sends one POST; show actual step/status, no invented percentage; follow at
one second while visible, abort requests/stop polling when hidden or unmounted.
The bounded backend job continues. On success reload tracks and select the new
track; show the signed timing adjustment and that the original remains available.
Show missing/failed jobs honestly and allow explicit retry. Do not queue another
job just because a view is reopened. Missing tooling disables Auto-align with
the backend's reason, while manual offset still works.

Manual offset is browser-local and independent from party playback. The parent
implements `Subtitles.delay` (reactive seconds) and `setDelay(seconds)`. Positive
means captions later, negative earlier. Values are bounded to ±600 seconds and
remembered per item/track in a bounded preference map. UI should use ±0.1 second
buttons, a numeric seconds field, and Reset. Disable while Off/unsupported; keep
keyboard input from triggering player shortcuts. HLS seek offsets and personal
subtitle delay are combined by the existing VTT renderer, not by seeking video.
