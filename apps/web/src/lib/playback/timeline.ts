import type { PlaybackSession } from "$lib/api/playback-types";

// The player always reasons in absolute item time. An HLS session starting at
// `start_seconds` reports media time 0 for that point, so the offset converts
// between the two. Direct play serves the original file, whose media time is
// already absolute.

export type TimelineSession = Pick<PlaybackSession, "method" | "start_seconds">;

export function mediaOffset(session: TimelineSession): number {
  return session.method === "direct" ? 0 : Math.max(0, session.start_seconds);
}

export function toAbsolute(session: TimelineSession, mediaTime: number): number {
  return mediaOffset(session) + (Number.isFinite(mediaTime) ? Math.max(0, mediaTime) : 0);
}

export function toMedia(session: TimelineSession, absolute: number): number {
  return Math.max(0, absolute - mediaOffset(session));
}

/** Keep seeks inside the item, leaving a little room before the end. */
export function clampTarget(target: number, duration: number | null): number {
  const safe = Number.isFinite(target) ? Math.max(0, target) : 0;
  if (!duration || duration <= 0) return safe;
  return Math.min(safe, Math.max(duration - 0.5, 0));
}

export interface MediaRange {
  start: number;
  end: number;
}

/** Media seconds a seek may land inside without leaving the produced window. */
const WINDOW_SLACK = 1;

export type SeekPlan = { kind: "media"; mediaTime: number } | { kind: "session"; absolute: number };

/**
 * Decide how to reach an absolute target. A direct stream can seek anywhere
 * with byte ranges. An HLS stream can only seek inside segments the server
 * has produced; anything before the session start or past the produced window
 * needs a new session starting at the target.
 */
export function planSeek(
  session: TimelineSession,
  target: number,
  seekable: MediaRange | null,
  duration: number | null,
): SeekPlan {
  const absolute = clampTarget(target, duration);
  if (session.method === "direct") return { kind: "media", mediaTime: absolute };
  const local = absolute - mediaOffset(session);
  if (
    seekable &&
    local >= Math.max(0, seekable.start - 0.25) &&
    local <= seekable.end - WINDOW_SLACK
  ) {
    return { kind: "media", mediaTime: local };
  }
  return { kind: "session", absolute };
}

/** Last range from `video.seekable`, or null when nothing is seekable yet. */
export function seekableRange(ranges: TimeRanges | null | undefined): MediaRange | null {
  if (!ranges || ranges.length === 0) return null;
  return { start: ranges.start(0), end: ranges.end(ranges.length - 1) };
}
