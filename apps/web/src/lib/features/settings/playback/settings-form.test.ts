import { describe, expect, test } from "vite-plus/test";
import type { Capabilities, Settings } from "$lib/api/admin-types";
import { hasSettingsErrors, settingsEqual, validateSettings } from "./settings-form";

const defaults: Settings = {
  transcoding_enabled: true,
  hardware_acceleration: "nvenc",
  hardware_encoding: true,
  hardware_decoding: false,
  gpu_index: 0,
  allow_cpu_fallback: true,
  max_concurrent_transcodes: 2,
  max_video_bitrate_mbps: 8,
  max_height: 1080,
  cpu_crf: 23,
  scratch_limit_gb: 10,
  session_idle_seconds: 120,
  resume_min_percent: 5,
  resume_max_percent: 90,
  resume_min_duration_seconds: 300,
};

const caps: Capabilities = {
  ffmpeg_available: true,
  nvenc_available: true,
  hardware_decode_available: false,
  ffmpeg_path: "/usr/bin/ffmpeg",
  scratch_path: "/var/cache/oxevu",
  notes: [],
};

describe("playback settings validation", () => {
  test("the documented defaults are valid", () => {
    expect(hasSettingsErrors(validateSettings(defaults, caps))).toBe(false);
  });

  test("rejects out-of-range, fractional and empty numbers", () => {
    const errors = validateSettings(
      { ...defaults, max_concurrent_transcodes: 0, cpu_crf: 22.5, max_height: Number.NaN },
      caps,
    );
    expect(errors.max_concurrent_transcodes).toBe("Use 1–16.");
    expect(errors.cpu_crf).toBe("Use a whole number.");
    expect(errors.max_height).toBe("Enter a number.");
  });

  test("resume window must be ordered", () => {
    expect(
      validateSettings({ ...defaults, resume_min_percent: 95 }, caps).resume_max_percent,
    ).toMatch(/higher/);
    expect(
      validateSettings({ ...defaults, resume_min_percent: 100 }, caps).resume_min_percent,
    ).toMatch(/less than 100/);
  });

  test("hardware decoding needs a detected decoder", () => {
    expect(
      validateSettings({ ...defaults, hardware_decoding: true }, caps).hardware_decoding,
    ).toBeDefined();
    const ok = validateSettings(
      { ...defaults, hardware_decoding: true },
      { ...caps, hardware_decode_available: true },
    );
    expect(ok.hardware_decoding).toBeUndefined();
  });

  test("detects unsaved changes", () => {
    expect(settingsEqual(defaults, { ...defaults })).toBe(true);
    expect(settingsEqual(defaults, { ...defaults, gpu_index: 1 })).toBe(false);
  });
});
