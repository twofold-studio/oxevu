import { describe, expect, test } from "vite-plus/test";
import type { AcquisitionConfig } from "$lib/api/acquisition-types";
import { validateAcquisition } from "./acquisition-form";

const valid: AcquisitionConfig = {
  enabled: false,
  download_dir: "/srv/downloads",
  listen_port: 6882,
  bind_interface: null,
  dht: true,
  max_active_downloads: 3,
  download_limit_kib: 0,
  upload_limit_kib: 0,
  min_free_gb: 20,
  storage_limit_gb: 500,
  seed_ratio: 1,
  seed_hours: 0,
  search_interval_minutes: 60,
  import_enabled: false,
  copy_fallback: false,
};

describe("acquisition settings validation", () => {
  test("accepts the defaults", () => {
    expect(validateAcquisition(valid)).toEqual({});
  });

  test("reports each invalid field on its own key", () => {
    const errors = validateAcquisition({
      ...valid,
      download_dir: "downloads",
      listen_port: 70_000,
      max_active_downloads: 1.5,
      upload_limit_kib: 8,
      search_interval_minutes: Number.NaN,
    });
    expect(Object.keys(errors).sort()).toEqual([
      "download_dir",
      "listen_port",
      "max_active_downloads",
      "search_interval_minutes",
      "upload_limit_kib",
    ]);
  });

  test("speed limits are zero or at least 16 KiB/s", () => {
    expect(validateAcquisition({ ...valid, download_limit_kib: 16 })).toEqual({});
    const errors = validateAcquisition({ ...valid, download_limit_kib: 15 });
    expect(errors.download_limit_kib).toBeTruthy();
  });
});
