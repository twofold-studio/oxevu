import * as sdk from "./generated/sdk.gen";
import { sdkClient, sdkData } from "./sdk";
import type * as A from "./acquisition-types";
import type { DiscoverFilters } from "$lib/media/discover-filters";

/** Validate the outer shape of foreign exports; the server validates each vendor's rows. */
export function externalExport(value: unknown): import("./generated/types.gen").ExternalExport {
  const record = (row: unknown): row is Record<string, unknown> =>
    row !== null && typeof row === "object" && !Array.isArray(row);
  if (Array.isArray(value) && value.every(record)) return value;
  if (record(value) && Array.isArray(value.results) && value.results.every(record)) {
    return { results: value.results };
  }
  throw new Error("Export must contain an array of records or a results object.");
}

const options = () => ({ client: sdkClient(), throwOnError: true as const });
export const acquisitionApi = {
  previewMigration: (body: A.MigrationInput) =>
    sdkData(sdk.migrationPreview({ ...options(), body })),
  migrate: (body: A.MigrationInput) => sdkData(sdk.migrationApply({ ...options(), body })),
  previewAdoption: (id: string, source_root: string) =>
    sdkData(sdk.adoptionPreview({ ...options(), path: { id }, body: { source_root } })),
  adopt: (id: string, source_root: string, token: string) =>
    sdkData(sdk.adoptionApply({ ...options(), path: { id }, body: { source_root, token } })),
  discover: (query: string, kind: A.MediaKind, page = 1, filters: DiscoverFilters = {}, signal?: AbortSignal) =>
    sdkData(sdk.discover({ ...options(), query: { query, kind, page, ...filters }, signal })),
  genres: (kind: A.MediaKind) => sdkData(sdk.genres({ ...options(), query: { kind } })),
  title: (kind: A.MediaKind, id: number) =>
    sdkData(sdk.titleDetails({ ...options(), path: { kind, id } })),
  trailer: (kind: A.MediaKind, id: number, signal: AbortSignal) =>
    sdkData(sdk.titleTrailer({ ...options(), path: { kind, id }, signal })),
  requests: () => sdkData(sdk.requestsList(options())),
  request: (title: A.DiscoverTitle, scope: A.RequestScope) =>
    sdkData(
      sdk.requestCreate({
        ...options(),
        body: { kind: title.kind, provider_id: title.provider_id, scope },
      }),
    ),
  decide: (id: string, status: string, note = "") =>
    sdkData(sdk.requestDecide({ ...options(), path: { id }, body: { status, note } })),
  monitored: () => sdkData(sdk.monitoredList(options())),
  monitor: (id: string, value: A.Monitor) =>
    sdkData(
      sdk.monitorPut({
        ...options(),
        path: { id },
        body: {
          enabled: value.enabled,
          scope: value.scope,
          library_id: value.library_id,
          root: value.root,
          profile_id: value.profile_id,
          automatic: value.automatic,
        },
      }),
    ),
  releases: (
    id: string,
    body: import("./generated/types.gen").ReleaseSearch,
    signal?: AbortSignal,
  ) => sdkData(sdk.releasesSearch({ ...options(), path: { id }, body, signal })),
  releaseSources: () => sdkData(sdk.releaseSources(options())),
  downloads: () => sdkData(sdk.downloadsList(options())),
  download: (body: A.DownloadInput) => sdkData(sdk.downloadCreate({ ...options(), body })),
  downloadStatus: (id: string) => sdkData(sdk.downloadGet({ ...options(), path: { id } })),
  downloadLimit: (id: string, download_limit_kib: number) =>
    sdkData(sdk.downloadLimit({ ...options(), path: { id }, body: { download_limit_kib } })),
  control: (id: string, action: string, selected_files?: number[]) =>
    sdkData(sdk.downloadControl({ ...options(), path: { id }, body: { action, selected_files } })),
  previewImport: (id: string, body: A.ImportInput) =>
    sdkData(sdk.importPreview({ ...options(), path: { id }, body })),
  import: (id: string, body: A.ImportInput) =>
    sdkData(sdk.importApply({ ...options(), path: { id }, body })),
  importDefaults: () => sdkData(sdk.importDefaults(options())),
  identify: (id: string, kind: A.MediaKind, provider_id: number) =>
    sdkData(sdk.downloadIdentity({ ...options(), path: { id }, body: { kind, provider_id } })),
  settings: () => sdkData(sdk.acquisitionSettingsGet(options())),
  configure: (body: A.AcquisitionConfig) =>
    sdkData(sdk.acquisitionSettingsPut({ ...options(), body })),
  profiles: () => sdkData(sdk.profilesList(options())),
  profile: (id: string, body: A.AcquisitionProfile) =>
    sdkData(sdk.profilePut({ ...options(), path: { id }, body })),
  indexers: () => sdkData(sdk.indexersList(options())),
  indexer: (
    id: string,
    body: Pick<A.Indexer, "name" | "endpoint" | "enabled" | "download_hosts"> & {
      api_key?: string;
    },
  ) => sdkData(sdk.indexerPut({ ...options(), path: { id }, body })),
  testIndexer: (id: string) => sdkData(sdk.indexerTest({ ...options(), path: { id } })),
  permissions: (id: string, body: A.AcquisitionPermissions) =>
    sdkData(sdk.accessPut({ ...options(), path: { id }, body })),
  events: () => sdkData(sdk.eventsList(options())),
};
