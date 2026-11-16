import { beforeEach, describe, expect, it, vi } from "vite-plus/test";
import {
  readVolume,
  saveVolume,
  readSubtitlePreference,
  saveSubtitlePreference,
} from "./personal-preferences";

beforeEach(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
  });
});

describe("personal player preferences", () => {
  it("remembers volume and mute across titles", () => {
    saveVolume(0.25, true);
    expect(readVolume()).toEqual({ level: 0.25, muted: true });
  });
  it("rejects invalid stored volume", () => {
    localStorage.setItem("oxevu.player.volume", '{"level":5,"muted":false}');
    expect(readVolume()).toEqual({ level: 1, muted: false });
  });
  it("remembers language and explicit subtitles off independently", () => {
    saveSubtitlePreference({ language: "en", label: "English" });
    expect(readSubtitlePreference()).toEqual({ language: "en", label: "English" });
    saveSubtitlePreference({ off: true });
    expect(readSubtitlePreference()).toEqual({ off: true });
  });
  it("tolerates malformed or unavailable browser storage", () => {
    localStorage.setItem("oxevu.player.subtitles", "not JSON");
    expect(readSubtitlePreference()).toBeNull();
    vi.stubGlobal("localStorage", {
      getItem() {
        throw new Error("blocked");
      },
      setItem() {
        throw new Error("blocked");
      },
    });
    expect(readVolume()).toEqual({ level: 1, muted: false });
    expect(() => saveVolume(0.5, false)).not.toThrow();
  });
});
