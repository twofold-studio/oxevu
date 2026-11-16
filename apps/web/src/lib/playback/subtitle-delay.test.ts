import { beforeEach, describe, expect, it, vi } from "vite-plus/test";
import { normalizeSubtitleDelay, readSubtitleDelay, saveSubtitleDelay } from "./subtitle-delay";

beforeEach(() => {
  const store = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => store.set(key, value),
  });
});

describe("personal subtitle delay", () => {
  it("keeps offsets separate by title and track and resets them to zero", () => {
    saveSubtitleDelay("movie", "english", 1.2);
    saveSubtitleDelay("movie", "japanese", -0.5);
    expect(readSubtitleDelay("movie", "english")).toBe(1.2);
    expect(readSubtitleDelay("movie", "japanese")).toBe(-0.5);
    expect(readSubtitleDelay("another", "english")).toBe(0);
    saveSubtitleDelay("movie", "english", 0);
    expect(readSubtitleDelay("movie", "english")).toBe(0);
  });
  it("bounds offsets and tolerates blocked storage", () => {
    expect(normalizeSubtitleDelay(900)).toBe(600);
    expect(normalizeSubtitleDelay(-900)).toBe(-600);
    expect(normalizeSubtitleDelay(NaN)).toBe(0);
    vi.stubGlobal("localStorage", {
      getItem() {
        throw Error("blocked");
      },
      setItem() {
        throw Error("blocked");
      },
    });
    expect(readSubtitleDelay("i", "t")).toBe(0);
    expect(() => saveSubtitleDelay("i", "t", 1)).not.toThrow();
  });
});
