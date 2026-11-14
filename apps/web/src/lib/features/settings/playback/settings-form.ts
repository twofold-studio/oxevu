import type { Capabilities, Settings } from "$lib/api/admin-types";

// Quick client feedback mirroring the server's validation ranges. The server
// stays authoritative and its message is shown if it still refuses.

type NumericKey = {
  [K in keyof Settings]: Settings[K] extends number ? K : never;
}[keyof Settings];

interface Range {
  min: number;
  max: number;
  integer?: boolean;
  /** Exclusive upper bound. */
  below?: boolean;
}

export const RANGES: Record<NumericKey, Range> = {
  gpu_index: { min: 0, max: 31, integer: true },
  max_concurrent_transcodes: { min: 1, max: 16, integer: true },
  max_video_bitrate_mbps: { min: 0.25, max: 200 },
  max_height: { min: 144, max: 4320, integer: true },
  cpu_crf: { min: 0, max: 51, integer: true },
  scratch_limit_gb: { min: 0.1, max: 1024 },
  session_idle_seconds: { min: 30, max: 3600, integer: true },
  resume_min_percent: { min: 0, max: 100, below: true },
  resume_max_percent: { min: 1, max: 100 },
  resume_min_duration_seconds: { min: 0, max: 86_400, integer: true },
};

export type SettingsErrors = Partial<Record<keyof Settings, string>>;

function checkNumber(value: unknown, range: Range): string | null {
  if (typeof value !== "number" || !Number.isFinite(value)) return "Enter a number.";
  if (range.integer && !Number.isInteger(value)) return "Use a whole number.";
  const tooHigh = range.below ? value >= range.max : value > range.max;
  if (value < range.min || tooHigh) {
    return range.below
      ? `Use at least ${range.min} and less than ${range.max}.`
      : `Use ${range.min}–${range.max}.`;
  }
  return null;
}

export function validateSettings(
  settings: Settings,
  capabilities: Capabilities | null,
): SettingsErrors {
  const errors: SettingsErrors = {};
  for (const [key, range] of Object.entries(RANGES) as [NumericKey, Range][]) {
    const message = checkNumber(settings[key], range);
    if (message) errors[key] = message;
  }
  if (
    !errors.resume_min_percent &&
    !errors.resume_max_percent &&
    settings.resume_min_percent >= settings.resume_max_percent
  ) {
    errors.resume_max_percent = "Must be higher than the minimum.";
  }
  if (settings.hardware_decoding && !capabilities?.hardware_decode_available) {
    errors.hardware_decoding = "The server reports no hardware decoder.";
  }
  return errors;
}

export const hasSettingsErrors = (errors: SettingsErrors) => Object.keys(errors).length > 0;

/** Field-by-field comparison for the unsaved-changes bar. */
export function settingsEqual(a: Settings, b: Settings): boolean {
  return (Object.keys(a) as (keyof Settings)[]).every((key) => a[key] === b[key]);
}
