import { describe, expect, it, vi } from "vite-plus/test";
import { ApiError } from "$lib/api/client";
import type { PlaybackSession, PlaybackSessionRequest } from "$lib/api/playback-types";
import { SessionController, SessionFailedError, type SessionApi } from "./session-controller";

const request: PlaybackSessionRequest = {
  item_id: "item",
  start_seconds: 0,
  mode: "auto",
  direct_play_supported: true,
};

function session(id: string, status: PlaybackSession["status"] = "ready"): PlaybackSession {
  return {
    id,
    item_id: "item",
    method: "nvenc",
    status,
    stream_url: status === "ready" ? `/api/v1/playback/sessions/${id}/hls/main.m3u8` : null,
    start_seconds: 0,
    duration_seconds: 100,
    reason: null,
    error: status === "failed" ? "ffmpeg exited" : null,
  };
}

/** A fake API whose create calls resolve when the test says so. */
function fakeApi() {
  const pending: ((s: PlaybackSession) => void)[] = [];
  const stopped: string[] = [];
  const get = vi.fn(async (id: string) => session(id));
  const api: SessionApi = {
    create: () => new Promise((resolve) => pending.push(resolve)),
    get,
    heartbeat: vi.fn(async () => undefined),
    stop: vi.fn(async (id: string) => {
      stopped.push(id);
    }),
  };
  return { api, get, pending, stopped };
}

const timing = { pollMs: 1, heartbeatMs: 1000, startTimeoutMs: 50 };
const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("SessionController", () => {
  it("coalesces queued opens to the latest target", async () => {
    const { api, pending } = fakeApi();
    const controller = new SessionController(api, () => undefined, timing);
    const first = controller.open(request);
    const second = controller.open({ ...request, start_seconds: 50 });
    await flush();
    expect(pending).toHaveLength(1);
    pending[0](session("b"));
    expect(await first).toBeNull();
    expect((await second)?.id).toBe("b");
    controller.dispose();
  });

  it("waits for stop acknowledgement before creating a replacement", async () => {
    const { api, pending } = fakeApi();
    let stopped!: () => void;
    api.stop = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          stopped = resolve;
        }),
    );
    const controller = new SessionController(api, () => undefined, timing);
    const first = controller.open(request);
    await flush();
    pending[0](session("a"));
    await first;
    const replacement = controller.open(request);
    await flush();
    expect(stopped).toBeTypeOf("function");
    expect(pending).toHaveLength(1);
    stopped();
    await flush();
    expect(pending).toHaveLength(2);
    pending[1](session("b"));
    await replacement;
    controller.dispose();
    await flush();
    stopped();
  });

  it("stops superseded starting sessions before creating the next", async () => {
    const { api, pending, stopped } = fakeApi();
    let wake!: () => void;
    const controller = new SessionController(
      api,
      () => undefined,
      timing,
      () =>
        new Promise<void>((resolve) => {
          wake = resolve;
        }),
    );
    const first = controller.open(request);
    await flush();
    pending[0](session("a", "starting"));
    await flush();
    const replacement = controller.open({ ...request, start_seconds: 50 });
    await flush();
    expect(stopped).toEqual(["a"]);
    expect(pending).toHaveLength(1);
    wake();
    await first;
    await flush();
    pending[1](session("b"));
    await replacement;
    controller.dispose();
  });

  it("does not create another transcode when stopping failed", async () => {
    const { api, pending } = fakeApi();
    api.stop = vi.fn(async () => {
      throw new Error("network unavailable");
    });
    const controller = new SessionController(api, () => undefined, timing);
    const first = controller.open(request);
    await flush();
    pending[0](session("a"));
    await first;
    await expect(controller.open(request)).rejects.toThrow("network unavailable");
    expect(pending).toHaveLength(1);
    controller.dispose();
  });

  it("polls a starting session until it is ready", async () => {
    const { api, get, pending } = fakeApi();
    const controller = new SessionController(api, () => undefined, timing);
    const opened = controller.open(request);
    await flush();
    pending[0](session("a", "starting"));
    expect((await opened)?.status).toBe("ready");
    expect(get).toHaveBeenCalledWith("a");
    controller.dispose();
  });

  it("reports failed sessions with the server's error and stops them", async () => {
    const { api, pending, stopped } = fakeApi();
    const controller = new SessionController(api, () => undefined, timing);
    const opened = controller.open(request);
    await flush();
    pending[0](session("a", "failed"));
    await expect(opened).rejects.toBeInstanceOf(SessionFailedError);
    expect(stopped).toEqual(["a"]);
  });

  it("times out sessions that never become ready", async () => {
    const { api, pending, stopped } = fakeApi();
    api.get = async (id) => session(id, "starting");
    const controller = new SessionController(api, () => undefined, timing);
    const opened = controller.open(request);
    await flush();
    pending[0](session("a", "starting"));
    await expect(opened).rejects.toThrow(/too long/);
    expect(stopped).toContain("a");
  });

  it("stops in-flight and active sessions on dispose", async () => {
    const { api, pending, stopped } = fakeApi();
    const controller = new SessionController(api, () => undefined, timing);
    const opened = controller.open(request);
    await flush();
    controller.dispose(true);
    await flush();
    pending[0](session("a"));
    expect(await opened).toBeNull();
    expect(stopped).toEqual(["a"]);
    expect(await controller.open(request)).toBeNull();
  });

  it("heartbeats and reports a reaped session once", async () => {
    vi.useFakeTimers();
    try {
      const { api, pending } = fakeApi();
      api.heartbeat = vi.fn(async () => {
        throw new ApiError(404, "gone");
      });
      const lost = vi.fn();
      const controller = new SessionController(api, lost, timing);
      const opened = controller.open(request);
      await vi.advanceTimersByTimeAsync(0);
      pending[0](session("a"));
      await opened;
      await vi.advanceTimersByTimeAsync(3000);
      expect(lost).toHaveBeenCalledTimes(1);
      expect(controller.session).toBeNull();
      controller.dispose();
    } finally {
      vi.useRealTimers();
    }
  });
});
