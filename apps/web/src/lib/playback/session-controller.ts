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
  pollMs: 500,
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
  async open(body: PlaybackSessionRequest): Promise<PlaybackSession | null> {
    if (this.disposed) return null;
    const generation = ++this.generation;
    this.release(false);

    let session = await this.api.create(body);
    const deadline = this.now() + this.timing.startTimeoutMs;
    while (session.status === "starting") {
      if (this.stale(generation)) return this.discard(session);
      if (this.now() >= deadline) {
        this.discard(session);
        throw new SessionFailedError("The server took too long to start the stream.", session);
      }
      await this.sleep(this.timing.pollMs);
      if (this.stale(generation)) return this.discard(session);
      session = await this.api.get(session.id);
    }
    if (this.stale(generation)) return this.discard(session);
    if (session.status === "failed" || !session.stream_url) {
      this.discard(session);
      throw new SessionFailedError(
        session.error ?? "The server couldn't start this stream.",
        session,
      );
    }
    this.active = session;
    this.startHeartbeat(session);
    return session;
  }

  /** Stop the current session; `keepalive` lets the request outlive the page. */
  release(keepalive: boolean) {
    clearInterval(this.heartbeat);
    this.heartbeat = undefined;
    const session = this.active;
    this.active = null;
    if (session) void this.api.stop(session.id, keepalive).catch(() => undefined);
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

  private discard(session: PlaybackSession): null {
    void this.api.stop(session.id).catch(() => undefined);
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
