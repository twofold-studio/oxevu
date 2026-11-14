import type { Party } from "$lib/api/playback-types";

/** Where the host's timeline is right now, from a snapshot and the server clock. */
export function expectedPosition(
  party: Pick<Party, "paused" | "position_seconds" | "updated_at_ms">,
  serverNowMs: number,
  duration: number | null,
): number {
  const elapsed = party.paused ? 0 : Math.max(0, serverNowMs - party.updated_at_ms) / 1000;
  const position = party.position_seconds + elapsed;
  return duration && duration > 0 ? Math.min(position, duration) : position;
}

/** Only newer revisions change state; equal ones refresh the clock only. */
export function isNewer(revision: number, seen: number | null): boolean {
  return seen === null || revision > seen;
}

/** Thresholds in seconds. Below `ignore`, drift is jitter we leave alone. */
export const DRIFT = {
  ignore: 0.35,
  /** Up to this, nudge the playback rate instead of seeking. */
  nudge: 2,
  rateDelta: 0.06,
} as const;

export type Correction =
  | { kind: "none" }
  | { kind: "rate"; rate: number }
  | { kind: "seek"; position: number };

/**
 * Decide how a playing follower should catch up. Small drift is ignored to
 * avoid microseeks; moderate drift is corrected smoothly with a slightly
 * faster or slower rate; large drift seeks. `driftSeconds` is local minus
 * expected: positive means this player is ahead.
 */
export function correctionFor(driftSeconds: number, expected: number): Correction {
  const size = Math.abs(driftSeconds);
  if (!Number.isFinite(driftSeconds) || size < DRIFT.ignore) return { kind: "none" };
  if (size <= DRIFT.nudge) {
    return { kind: "rate", rate: driftSeconds > 0 ? 1 - DRIFT.rateDelta : 1 + DRIFT.rateDelta };
  }
  return { kind: "seek", position: expected };
}

/** While paused, any visible difference is fixed with one seek. */
export function pausedCorrection(driftSeconds: number, expected: number): Correction {
  return Math.abs(driftSeconds) > 0.5 ? { kind: "seek", position: expected } : { kind: "none" };
}
