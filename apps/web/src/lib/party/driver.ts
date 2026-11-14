import type { Party } from "$lib/api/playback-types";
import { correctionFor, expectedPosition, pausedCorrection } from "./sync";

/** The parts of PlaybackEngine the sync driver steers. */
export interface SyncTarget {
  readonly phase: string;
  readonly pendingTarget: number | null;
  readonly absolute: number;
  readonly paused: boolean;
  readonly autoplayBlocked: boolean;
  play(): Promise<boolean>;
  pause(): void;
  seek(position: number): void;
  setRate(rate: number): void;
}

export type SyncState =
  | { kind: "loading" }
  | { kind: "blocked" }
  | { kind: "in-sync"; drift: number }
  | { kind: "correcting"; drift: number };

/** After a seek, let buffering settle before judging drift again. */
const SEEK_COOLDOWN_MS = 3000;
/** After the host acts locally, wait for the server to echo the change. */
const HOST_INTENT_MS = 2000;

/**
 * Applies the server timeline to one local player. Everyone, the host
 * included, follows the server: the host's own actions are applied locally
 * at once and sent as controls; the echo then matches. Corrections are made
 * programmatically and never sent back, so they can't loop.
 */
export class SyncDriver {
  private lastSeekAt = -Infinity;
  private intentUntil = -Infinity;

  constructor(private readonly target: SyncTarget) {}

  /** The host just played, paused or seeked locally. */
  noteLocalIntent(nowMs: number) {
    this.intentUntil = nowMs + HOST_INTENT_MS;
  }

  apply(party: Party, serverNowMs: number, nowMs: number, duration: number | null): SyncState {
    const target = this.target;
    if (target.phase !== "ready" || target.pendingTarget !== null) return { kind: "loading" };
    const expected = expectedPosition(party, serverNowMs, duration);
    const drift = target.absolute - expected;
    if (nowMs < this.intentUntil) return { kind: "in-sync", drift };

    if (party.paused) {
      if (!target.paused) target.pause();
      target.setRate(1);
      return this.correct(pausedCorrection(drift, expected), drift, nowMs);
    }

    if (target.autoplayBlocked) return { kind: "blocked" };
    if (target.paused) void target.play();
    return this.correct(correctionFor(drift, expected), drift, nowMs);
  }

  private correct(
    correction: ReturnType<typeof correctionFor>,
    drift: number,
    nowMs: number,
  ): SyncState {
    switch (correction.kind) {
      case "none":
        this.target.setRate(1);
        return { kind: "in-sync", drift };
      case "rate":
        this.target.setRate(correction.rate);
        return { kind: "correcting", drift };
      case "seek":
        this.target.setRate(1);
        if (nowMs - this.lastSeekAt >= SEEK_COOLDOWN_MS) {
          this.lastSeekAt = nowMs;
          this.target.seek(correction.position);
        }
        return { kind: "correcting", drift };
    }
  }
}
