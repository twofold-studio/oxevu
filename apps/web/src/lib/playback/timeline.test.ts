import { describe, expect, it } from "vite-plus/test";
import { clampTarget, mediaOffset, planSeek, toAbsolute, toMedia } from "./timeline";

const hls = { method: "nvenc" as const, start_seconds: 600 };
const direct = { method: "direct" as const, start_seconds: 600 };

describe("session offsets", () => {
  it("offsets HLS media time by the session start", () => {
    expect(mediaOffset(hls)).toBe(600);
    expect(toAbsolute(hls, 12.5)).toBe(612.5);
    expect(toMedia(hls, 612.5)).toBe(12.5);
  });

  it("treats direct media time as absolute", () => {
    expect(mediaOffset(direct)).toBe(0);
    expect(toAbsolute(direct, 612.5)).toBe(612.5);
  });

  it("never yields negative or NaN positions", () => {
    expect(toAbsolute(hls, Number.NaN)).toBe(600);
    expect(toMedia(hls, 10)).toBe(0);
  });
});

describe("clampTarget", () => {
  it("keeps targets inside the item", () => {
    expect(clampTarget(-5, 100)).toBe(0);
    expect(clampTarget(200, 100)).toBe(99.5);
    expect(clampTarget(40, null)).toBe(40);
  });
});

describe("planSeek", () => {
  const window = { start: 0, end: 120 };

  it("seeks direct streams in place anywhere", () => {
    expect(planSeek(direct, 30, null, 3600)).toEqual({ kind: "media", mediaTime: 30 });
  });

  it("seeks inside the produced HLS window locally", () => {
    expect(planSeek(hls, 660, window, 3600)).toEqual({ kind: "media", mediaTime: 60 });
  });

  it("needs a new session before the session start", () => {
    expect(planSeek(hls, 300, window, 3600)).toEqual({ kind: "session", absolute: 300 });
  });

  it("needs a new session past the produced window", () => {
    expect(planSeek(hls, 2000, window, 3600)).toEqual({ kind: "session", absolute: 2000 });
  });

  it("needs a new session when nothing is seekable yet", () => {
    expect(planSeek(hls, 610, null, 3600)).toEqual({ kind: "session", absolute: 610 });
  });

  it("clamps the absolute target to the item before planning", () => {
    expect(planSeek(hls, 9999, window, 3600)).toEqual({ kind: "session", absolute: 3599.5 });
  });
});
