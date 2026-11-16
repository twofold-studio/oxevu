import { describe, expect, it, vi } from "vite-plus/test";
import type { Party } from "$lib/api/playback-types";
import { SyncDriver, type SyncTarget } from "./driver";

function target(overrides: Partial<SyncTarget> = {}) {
  const t = {
    phase: "ready",
    pendingTarget: null,
    absolute: 100,
    paused: false,
    autoplayBlocked: false,
    play: vi.fn(async () => true),
    pause: vi.fn(),
    seek: vi.fn(),
    setRate: vi.fn(),
    ...overrides,
  };
  return t;
}

const party = (overrides: Partial<Party> = {}): Party => ({
  id: "p",
  name: "Movie night",
  host_user_id: "h",
  item_id: "i",
  paused: false,
  position_seconds: 100,
  updated_at_ms: 0,
  server_now_ms: 0,
  revision: 1,
  playback_revision: 1,
  members: [],
  ...overrides,
});

describe("SyncDriver", () => {
  it("leaves a player alone when it is within the jitter threshold", () => {
    const t = target({ absolute: 100.2 });
    const state = new SyncDriver(t).apply(party(), 0, 0, null);
    expect(state.kind).toBe("in-sync");
    expect(t.seek).not.toHaveBeenCalled();
    expect(t.setRate).toHaveBeenLastCalledWith(1);
  });

  it("nudges the rate for moderate drift", () => {
    const t = target({ absolute: 99 });
    new SyncDriver(t).apply(party(), 0, 0, null);
    expect(t.setRate).toHaveBeenLastCalledWith(1.06);
  });

  it("seeks once for large drift, then waits for the stream to settle", () => {
    const t = target({ absolute: 40 });
    const driver = new SyncDriver(t);
    // Server says 10 s have passed since the snapshot at position 100.
    driver.apply(party(), 10_000, 0, null);
    driver.apply(party(), 11_000, 1000, null);
    expect(t.seek).toHaveBeenCalledTimes(1);
    expect(t.seek).toHaveBeenCalledWith(110);
    driver.apply(party(), 14_000, 4000, null);
    expect(t.seek).toHaveBeenCalledTimes(2);
  });

  it("follows the host's pause and aligns the paused frame", () => {
    const t = target({ absolute: 120 });
    new SyncDriver(t).apply(party({ paused: true }), 50_000, 0, null);
    expect(t.pause).toHaveBeenCalled();
    expect(t.seek).toHaveBeenCalledWith(100);
  });

  it("resumes a paused follower when the host plays", () => {
    const t = target({ paused: true });
    new SyncDriver(t).apply(party(), 0, 0, null);
    expect(t.play).toHaveBeenCalled();
  });

  it("reports blocked autoplay instead of retrying play", () => {
    const t = target({ paused: true, autoplayBlocked: true });
    expect(new SyncDriver(t).apply(party(), 0, 0, null)).toEqual({ kind: "blocked" });
    expect(t.play).not.toHaveBeenCalled();
  });

  it("does not correct while a stream is starting", () => {
    const t = target({ phase: "starting", absolute: 0 });
    expect(new SyncDriver(t).apply(party(), 0, 0, null)).toEqual({ kind: "loading" });
    expect(t.seek).not.toHaveBeenCalled();
  });

  it("gives the host's local action time to round-trip before correcting", () => {
    const t = target({ absolute: 300 });
    const driver = new SyncDriver(t);
    driver.noteLocalIntent(0);
    driver.apply(party(), 0, 1000, null);
    expect(t.seek).not.toHaveBeenCalled();
    driver.apply(party(), 0, 2500, null);
    expect(t.seek).toHaveBeenCalledWith(100);
  });
});
