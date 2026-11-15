import { ApiError } from "$lib/api/client";
import type { PlaybackSession, PlaybackSessionRequest } from "$lib/api/playback-types";

export interface SessionApi {
  create(body: PlaybackSessionRequest): Promise<PlaybackSession>;
  get(id: string): Promise<PlaybackSession>;
  heartbeat(id: string): Promise<void>;
  stop(id: string, keepalive?: boolean): Promise<void>;
}

export interface SessionTiming {
  pollMs: number;
  heartbeatMs: number;
  /** Give up waiting for a `starting` session after this long. */
  startTimeoutMs: number;
}

export const defaultTiming: SessionTiming = {
  pollMs: 150,
  heartbeatMs: 15_000,
  startTimeoutMs: 90_000,
};

/** The server reported the session as failed, or it never became ready. */
export class SessionFailedError extends Error {
  readonly session: PlaybackSession | null;

  constructor(message: string, session: PlaybackSession | null) {
    super(message);
    this.name = "SessionFailedError";
    this.session = session;
  }
}

/**
 * Owns at most one server playback session at a time. Every `open` supersedes
 * the previous call: older sessions are stopped as soon as they are known,
 * even if their creation response arrives after a newer request, so rapid
 * seeks never leave transcodes running. Ready sessions are kept alive with
 * heartbeats until replaced or disposed.
 */
export class SessionController {
  private generation = 0;
  private active: PlaybackSession | null = null;
  private heartbeat: ReturnType<typeof setInterval> | undefined;
  private disposed = false;
  private queue: Promise<unknown> = Promise.resolve();
  private closing: Promise<void> = Promise.resolve();

  constructor(
    private readonly api: SessionApi,
    private readonly onLost: (session: PlaybackSession) => void,
    private readonly timing: SessionTiming = defaultTiming,
    private readonly sleep: (ms: number) => Promise<void> = (ms) =>
      new Promise((resolve) => setTimeout(resolve, ms)),
    private readonly now: () => number = () => Date.now(),
  ) {}

  get session(): PlaybackSession | null {
    return this.active;
  }

  /** Resolves with the ready session, or null when a newer call superseded it. */
  open(body: PlaybackSessionRequest): Promise<PlaybackSession | null> {
    if (this.disposed) return Promise.resolve(null);
    const generation = ++this.generation;
    this.release(false);
    const opened = this.queue.then(async () => {
      if (this.stale(generation)) return null;
      await this.closing;
      if (this.stale(generation)) return null;
      return this.start(body, generation);
    });
    // A failed create must not prevent a later explicit retry.
    this.queue = opened.catch(() => undefined);
    return opened;
  }

  private async start(
    body: PlaybackSessionRequest,
    generation: number,
  ): Promise<PlaybackSession | null> {
    let session = await this.api.create(body);
    // Track starting sessions too: cancellation must not wait for readiness.
    this.active = session;
    if (this.stale(generation)) return this.discard(session);
    const deadline = this.now() + this.timing.startTimeoutMs;
    try {
      while (session.status === "starting") {
        if (this.stale(generation)) return this.discard(session);
        if (this.now() >= deadline) {
          throw new SessionFailedError("The server took too long to start the stream.", session);
        }
        await this.sleep(this.timing.pollMs);
        if (this.stale(generation)) return this.discard(session);
        session = await this.api.get(session.id);
      }
      if (this.stale(generation)) return this.discard(session);
      if (session.status === "failed" || !session.stream_url) {
        throw new SessionFailedError(
          session.error ?? "The server couldn't start this stream.",
          session,
        );
      }
      this.active = session;
      this.startHeartbeat(session);
      return session;
    } catch (error) {
      await this.discard(session);
      throw error;
    }
  }

  /** Stop the current session; `keepalive` lets the request outlive the page. */
  release(keepalive: boolean) {
    clearInterval(this.heartbeat);
    this.heartbeat = undefined;
    const session = this.active;
    this.active = null;
    if (session) {
      this.closing = this.closing
        .then(() => this.api.stop(session.id, keepalive))
        .catch((error: unknown) => {
          if (error instanceof ApiError && (error.status === 404 || error.status === 410)) return;
          throw error;
        });
      // Teardown callers needn't await, but open always waits before creating.
      void this.closing.catch(() => undefined);
    }
  }

  /** Cancel pending opens and stop everything. Safe to call more than once. */
  dispose(keepalive = false) {
    this.disposed = true;
    this.generation++;
    this.release(keepalive);
  }

  private stale(generation: number): boolean {
    return this.disposed || generation !== this.generation;
  }

  private async discard(session: PlaybackSession): Promise<null> {
    if (this.active?.id === session.id) this.release(false);
    await this.closing;
    return null;
  }

  private startHeartbeat(session: PlaybackSession) {
    this.heartbeat = setInterval(() => {
      this.api.heartbeat(session.id).catch((error: unknown) => {
        // Transient failures retry on the next beat; a missing session has
        // been reaped (idle lease, restart) and must be recreated.
        const gone = error instanceof ApiError && (error.status === 404 || error.status === 410);
        if (gone && this.active?.id === session.id) {
          clearInterval(this.heartbeat);
          this.active = null;
          this.onLost(session);
        }
      });
    }, this.timing.heartbeatMs);
  }
}
