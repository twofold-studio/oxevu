import type { AcquisitionConfig } from "$lib/api/acquisition-types";

// Client feedback mirroring the server's acquisition ranges. The server stays
// authoritative; its message is shown if it still refuses a value.

export type AcquisitionErrors = Partial<Record<keyof AcquisitionConfig, string>>;

type NumericKey = {
  [K in keyof AcquisitionConfig]-?: AcquisitionConfig[K] extends number ? K : never;
}[keyof AcquisitionConfig];

type Range = { min: number; max: number; integer?: boolean };

const RANGES: Record<NumericKey, Range> = {
  listen_port: { min: 0, max: 65_535, integer: true },
  max_active_downloads: { min: 1, max: 16, integer: true },
  download_limit_kib: { min: 0, max: 1_000_000 },
  upload_limit_kib: { min: 0, max: 1_000_000 },
  storage_limit_gb: { min: 1, max: 100_000 },
  min_free_gb: { min: 0, max: 100_000 },
  seed_ratio: { min: 0, max: 100 },
  seed_hours: { min: 0, max: 87_600 },
  search_interval_minutes: { min: 5, max: 1440 },
};

export function validateAcquisition(config: AcquisitionConfig): AcquisitionErrors {
  const errors: AcquisitionErrors = {};
  if (!config.download_dir.startsWith("/")) errors.download_dir = "Use an absolute folder path.";
  for (const [key, range] of Object.entries(RANGES) as [NumericKey, Range][]) {
    const value = config[key];
    if (typeof value !== "number" || !Number.isFinite(value)) errors[key] = "Enter a number.";
    else if (range.integer && !Number.isInteger(value)) errors[key] = "Use a whole number.";
    else if (value < range.min || value > range.max) errors[key] = `Use ${range.min}–${range.max}.`;
  }
  for (const key of ["download_limit_kib", "upload_limit_kib"] as const) {
    const value = config[key];
    if (!errors[key] && value > 0 && value < 16)
      errors[key] = "Use zero for unlimited, or at least 16 KiB/s.";
  }
  return errors;
}
