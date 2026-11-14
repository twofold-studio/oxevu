import { describe, expect, it } from "vite-plus/test";
import { ClockEstimator } from "./clock";
import { correctionFor, expectedPosition, isNewer, pausedCorrection } from "./sync";

describe("ClockEstimator", () => {
  it("assumes the server stamped the midpoint of the round trip", () => {
    const clock = new ClockEstimator();
    // Local 1000 -> 1100; the server said 5050 at the midpoint (local 1050).
    clock.add(1000, 1100, 5050);
    expect(clock.best).toEqual({ offsetMs: 4000, rttMs: 100 });
    expect(clock.serverNow(2000)).toBe(6000);
  });

  it("trusts the fastest round trip", () => {
    const clock = new ClockEstimator();
    clock.add(0, 400, 1000); // offset 800, noisy
    clock.add(1000, 1020, 5010); // offset 4000, precise
    clock.add(2000, 2300, 6000); // offset 3850
    expect(clock.best?.offsetMs).toBe(4000);
  });

  it("rejects impossible samples", () => {
    const clock = new ClockEstimator();
    expect(clock.add(100, 50, 1000)).toBeNull();
    expect(clock.best).toBeNull();
    expect(clock.serverNow(10)).toBe(10);
  });
});

describe("expectedPosition", () => {
  const base = { position_seconds: 100, updated_at_ms: 10_000 };

  it("extrapolates a playing timeline by server time", () => {
    expect(expectedPosition({ ...base, paused: false }, 12_500, null)).toBe(102.5);
  });

  it("holds a paused timeline", () => {
    expect(expectedPosition({ ...base, paused: true }, 50_000, null)).toBe(100);
  });

  it("never runs backwards on clock skew or past the end", () => {
    expect(expectedPosition({ ...base, paused: false }, 9_000, null)).toBe(100);
    expect(expectedPosition({ ...base, paused: false }, 99_000, 120)).toBe(120);
  });
});

describe("revisions", () => {
  it("ignores stale and repeated revisions", () => {
    expect(isNewer(1, null)).toBe(true);
    expect(isNewer(5, 4)).toBe(true);
    expect(isNewer(4, 4)).toBe(false);
    expect(isNewer(3, 4)).toBe(false);
  });
});

describe("drift correction", () => {
  it("ignores jitter", () => {
    expect(correctionFor(0.2, 50)).toEqual({ kind: "none" });
    expect(correctionFor(-0.3, 50)).toEqual({ kind: "none" });
  });

  it("slows down when ahead and speeds up when behind", () => {
    expect(correctionFor(1, 50)).toEqual({ kind: "rate", rate: 0.94 });
    expect(correctionFor(-1, 50)).toEqual({ kind: "rate", rate: 1.06 });
  });

  it("seeks on large drift", () => {
    expect(correctionFor(-8, 50)).toEqual({ kind: "seek", position: 50 });
  });

  it("aligns paused followers with one seek", () => {
    expect(pausedCorrection(0.3, 10)).toEqual({ kind: "none" });
    expect(pausedCorrection(3, 10)).toEqual({ kind: "seek", position: 10 });
  });
});
