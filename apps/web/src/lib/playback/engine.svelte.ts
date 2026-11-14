import { playbackApi } from "$lib/api/endpoints";
import type { Item } from "$lib/api/types";
import type { PlaybackMode, PlaybackSession } from "$lib/api/playback-types";
import { directPlaySupported } from "./direct-support";
import { attachStream, prepareHls, type AttachedStream, type StreamFailure } from "./hls-attach";
import { explainMediaError, explainSessionError, type PlaybackProblem } from "./playback-error";
import { SessionController } from "./session-controller";
import { planSeek, seekableRange, toAbsolute, toMedia } from "./timeline";

export type EnginePhase = "idle" | "starting" | "ready" | "failed";

/** Seeks that need a new server session wait this long for the next seek. */
const SESSION_SEEK_DEBOUNCE_MS = 250;

const FALLBACK_NOTICE =
  "This browser couldn't play the original file, so Oxevu switched to a converted stream.";

/**
 * Drives one <video> from server playback sessions. Everything public is in
 * absolute item seconds; session offsets stay inside. One engine per item.
 */
export class PlaybackEngine {
  readonly item: Item;
  private readonly canTranscode: boolean;
  private readonly onSignedOut: () => void;

  phase = $state<EnginePhase>("idle");
  session = $state<PlaybackSession | null>(null);
  problem = $state<PlaybackProblem | null>(null);
  mode = $state<PlaybackMode>("auto");
  /** Explains an automatic change of delivery, such as the direct-play fallback. */
  notice = $state<string | null>(null);
  /** Absolute target while a new session is prepared; shown on the timeline. */
  pendingTarget = $state<number | null>(null);
  autoplayBlocked = $state(false);
  /** Media-element time, kept current by the player's timeupdate handler. */
  mediaTime = $state(0);

  readonly absolute = $derived(
    this.pendingTarget ?? (this.session ? toAbsolute(this.session, this.mediaTime) : 0),
  );

  /** Full item length in seconds, when the probe or the session knows it. */
  get duration(): number | null {
    return this.item.duration_seconds ?? this.session?.duration_seconds ?? null;
  }

  private video: HTMLVideoElement | null = null;
  private stream: AttachedStream | null = null;
  private readonly controller: SessionController;
  private directOk: boolean;
  private fellBack = false;
  private wantPlaying = true;
  private loadToken = 0;
  private seekTimer: ReturnType<typeof setTimeout> | undefined;
  private disposed = false;

  constructor(item: Item, canTranscode: boolean, onSignedOut: () => void) {
    this.item = item;
    this.canTranscode = canTranscode;
    this.onSignedOut = onSignedOut;
    this.directOk = directPlaySupported(item);
    this.controller = new SessionController(
      {
        create: async (body) => (await playbackApi.create(body)).session,
        get: async (id) => (await playbackApi.get(id)).session,
        heartbeat: (id) => playbackApi.heartbeat(id),
        stop: (id, keepalive) => playbackApi.stop(id, keepalive),
      },
      // Reaped by the server (idle lease, restart): continue from here.
      () => {
        if (!this.disposed) void this.load(this.absolute, this.video ? !this.video.paused : false);
      },
    );
  }

  attach(video: HTMLVideoElement) {
    this.video = video;
  }

  /** Start (or restart) a session at an absolute position. */
  async load(absolute: number, playing = this.wantPlaying) {
    if (this.disposed) return;
    const token = ++this.loadToken;
    clearTimeout(this.seekTimer);
    this.wantPlaying = playing;
    this.pendingTarget = absolute;
    this.phase = "starting";
    this.problem = null;
    // Drop the old stream now: its server session is about to be stopped.
    this.detach();

    let session: PlaybackSession | null;
    let video: HTMLVideoElement | null;
    try {
      session = await this.controller.open({
        item_id: this.item.id,
        start_seconds: Math.round(absolute * 1000) / 1000,
        mode: this.mode,
        direct_play_supported: this.directOk,
      });
      video = this.video;
      if (!session || token !== this.loadToken || !video) return;
      if (session.method !== "direct") await prepareHls(video);
      if (token !== this.loadToken) return;
      this.session = session;
      this.mediaTime = toMedia(session, absolute);
      // Synchronous after the last await, so concurrent loads can't interleave.
      this.stream = attachStream(
        video,
        session.stream_url ?? "",
        session.method !== "direct",
        (f) => this.streamFailed(f, token),
      );
    } catch (error) {
      if (token !== this.loadToken) return;
      this.fail(explainSessionError(error));
      return;
    }

    if (!(await waitForMetadata(video)) || token !== this.loadToken) return;
    const target = toMedia(session, absolute);
    if (target > 0.25) video.currentTime = target;
    this.mediaTime = video.currentTime;
    this.pendingTarget = null;
    this.phase = "ready";
    if (this.wantPlaying) await this.play();
  }

  async play(): Promise<boolean> {
    this.wantPlaying = true;
    if (!this.video || this.phase !== "ready") return false;
    try {
      await this.video.play();
      this.autoplayBlocked = false;
      return true;
    } catch (error) {
      // Browsers block unmuted autoplay without a user gesture; the player
      // offers an explicit play button instead.
      if (error instanceof DOMException && error.name === "NotAllowedError") {
        this.autoplayBlocked = true;
      }
      return false;
    }
  }

  pause() {
    this.wantPlaying = false;
    this.video?.pause();
  }

  get paused(): boolean {
    return this.video ? this.video.paused : !this.wantPlaying;
  }

  /** Seek to an absolute position, inside the stream or via a new session. */
  seek(target: number) {
    const video = this.video;
    if (!video || !this.session || this.phase !== "ready") {
      if (this.phase === "starting") void this.load(target);
      return;
    }
    // Some native HLS players report no seekable range for a growing
    // playlist; buffered media is still seekable.
    const range = seekableRange(video.seekable) ?? seekableRange(video.buffered);
    const plan = planSeek(this.session, target, range, this.duration);
    if (plan.kind === "media") {
      clearTimeout(this.seekTimer);
      this.pendingTarget = null;
      video.currentTime = plan.mediaTime;
      this.mediaTime = plan.mediaTime;
      return;
    }
    // Scrubbing produces bursts; only the last target starts a session.
    if (this.pendingTarget === null) this.wantPlaying = !video.paused;
    this.pendingTarget = plan.absolute;
    clearTimeout(this.seekTimer);
    this.seekTimer = setTimeout(() => void this.load(plan.absolute), SESSION_SEEK_DEBOUNCE_MS);
  }

  setRate(rate: number) {
    if (this.video && this.video.playbackRate !== rate) this.video.playbackRate = rate;
  }

  /** User-chosen delivery; applies from the current position. */
  setMode(mode: PlaybackMode) {
    if (mode === this.mode && this.phase !== "failed") return;
    this.mode = mode;
    this.notice = null;
    void this.load(this.absolute, this.video ? !this.video.paused : this.wantPlaying);
  }

  retry() {
    void this.load(this.absolute);
  }

  /** Called from the <video> error event. */
  async handleMediaError() {
    const video = this.video;
    const session = this.session;
    if (!video?.error || !session || !this.stream) return;
    const code = video.error.code;
    const unsupported =
      code === MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED || code === MediaError.MEDIA_ERR_DECODE;
    if (
      session.method === "direct" &&
      this.mode === "auto" &&
      unsupported &&
      this.canTranscode &&
      !this.fellBack
    ) {
      this.fellBack = true;
      this.directOk = false;
      this.notice = FALLBACK_NOTICE;
      void this.load(this.absolute);
      return;
    }
    const token = this.loadToken;
    const problem = await explainMediaError(
      video.error,
      session.stream_url ?? "",
      session.method !== "direct",
    );
    if (token === this.loadToken) this.fail(problem);
  }

  /** Stop everything. `keepalive` is for page unload. */
  dispose(keepalive = false) {
    this.disposed = true;
    this.loadToken++;
    clearTimeout(this.seekTimer);
    this.detach();
    this.controller.dispose(keepalive);
  }

  private streamFailed(failure: StreamFailure, token: number) {
    if (token !== this.loadToken) return;
    if (failure.status === 401) {
      this.fail({
        title: "Your session has ended",
        detail: "Sign in again to keep watching.",
        signedOut: true,
      });
      return;
    }
    this.fail({ title: "The stream stopped", detail: failure.message });
  }

  private fail(problem: PlaybackProblem) {
    this.problem = problem;
    this.phase = "failed";
    this.pendingTarget = null;
    this.detach();
    this.controller.release(false);
    if (problem.signedOut) this.onSignedOut();
  }

  private detach() {
    this.stream?.detach();
    this.stream = null;
  }
}

function waitForMetadata(video: HTMLVideoElement): Promise<boolean> {
  if (video.readyState >= HTMLMediaElement.HAVE_METADATA) return Promise.resolve(true);
  return new Promise((resolve) => {
    const done = (ok: boolean) => {
      video.removeEventListener("loadedmetadata", onMeta);
      video.removeEventListener("error", onError);
      resolve(ok);
    };
    const onMeta = () => done(true);
    // Errors reach handleMediaError through the player's error handler.
    const onError = () => done(false);
    video.addEventListener("loadedmetadata", onMeta);
    video.addEventListener("error", onError);
  });
}
