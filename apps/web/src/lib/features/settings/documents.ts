import { settingsApi } from "$lib/api/endpoints";
import { acquisitionApi } from "$lib/api/acquisition";
import type { AcquisitionConfig } from "$lib/api/acquisition-types";
import type { Settings, SettingsEnvelope } from "$lib/api/admin-types";
import { SettingsDocument } from "./document";

// Shared for the life of the app so every page that edits part of a record
// goes through the same write queue.

/** Playback conversion, limits and resume rules, plus probed capabilities. */
export const playbackDocument = new SettingsDocument<SettingsEnvelope, Settings>(
  () => settingsApi.get(),
  (value) => settingsApi.save(value),
  (envelope) => envelope.settings,
);

export interface AcquisitionEnvelope {
  settings: AcquisitionConfig;
  engine?: string;
}

/** Download engine, storage, network, queue, seeding and import rules. */
export const acquisitionDocument = new SettingsDocument<AcquisitionEnvelope, AcquisitionConfig>(
  () => acquisitionApi.settings(),
  (value) => acquisitionApi.configure(value),
  (envelope) => envelope.settings,
);
