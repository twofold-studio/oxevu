import { itemsApi } from "$lib/api/endpoints";
import { isFinished } from "$lib/media/progress";

/** Save at most this often while playing; pauses, seeks and exit save at once. */
const INTERVAL_SECONDS = 10;

export interface SavedProgress {
  position_seconds: number;
  completed: boolean;
}

/**
 * Writes playback position to PUT /items/{id}/progress. One request in flight
 * at a time; a newer position queued meanwhile is sent when it settles. The
 * final write on exit uses keepalive so it survives page unload.
 */
export class ProgressReporter {
  private lastSaved: SavedProgress | null = null;
  private lastSentAt = -Infinity;
  private inFlight: Promise<void> | null = null;
  private queued: SavedProgress | null = null;

  constructor(
    private readonly itemId: string,
    private readonly onSaved: (progress: SavedProgress) => void,
    private readonly onError: (error: unknown) => void,
  ) {}

  private snapshot(position: number, duration: number | null, ended: boolean): SavedProgress {
    return {
      position_seconds: Math.max(0, Math.round(position * 10) / 10),
      completed: ended || isFinished(position, duration),
    };
  }

  /** Call on timeupdate; sends only when the interval has elapsed. */
  tick(position: number, duration: number | null) {
    if (Math.abs(position - this.lastSentAt) < INTERVAL_SECONDS) return;
    this.save(position, duration);
  }

  /** Save now (pause, seek, end). */
  save(position: number, duration: number | null, ended = false) {
    if (!Number.isFinite(position)) return;
    const next = this.snapshot(position, duration, ended);
    if (
      this.lastSaved &&
      this.lastSaved.completed === next.completed &&
      Math.abs(this.lastSaved.position_seconds - next.position_seconds) < 1
    ) {
      return;
    }
    this.lastSentAt = position;
    if (this.inFlight) {
      this.queued = next;
      return;
    }
    this.send(next, false);
  }

  /** Best-effort final write when leaving the player or the page. */
  flush(position: number, duration: number | null, ended = false) {
    if (!Number.isFinite(position)) return;
    const next = this.snapshot(position, duration, ended);
    if (this.lastSaved && this.lastSaved.position_seconds === next.position_seconds) return;
    this.queued = null;
    this.send(next, true);
  }

  private send(progress: SavedProgress, keepalive: boolean) {
    this.inFlight = itemsApi
      .saveProgress(this.itemId, progress, keepalive)
      .then(() => {
        this.lastSaved = progress;
        this.onSaved(progress);
      })
      .catch((error: unknown) => this.onError(error))
      .finally(() => {
        this.inFlight = null;
        const queued = this.queued;
        this.queued = null;
        if (queued) this.send(queued, false);
      });
  }
}
