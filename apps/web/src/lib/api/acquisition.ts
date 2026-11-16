import { request } from "./client";
import type * as A from "./acquisition-types";
const path = (value: string) => encodeURIComponent(value);
const put = <T>(route: string, body: unknown) =>
  request<T>(`/acquisition${route}`, { method: "PUT", body });
const post = <T>(route: string, body?: unknown) =>
  request<T>(`/acquisition${route}`, { method: "POST", body });
const get = <T>(route: string) => request<T>(`/acquisition${route}`);

export const acquisitionApi = {
  previewMigration: (body: A.MigrationInput) =>
    post<A.MigrationPreview>("/migration/preview", body),
  migrate: (body: A.MigrationInput) =>
    post<{ results: { key: string; imported: boolean; error?: string }[] }>("/migration", body),
  previewAdoption: (id: string, source_root: string) =>
    post<{ files: A.ImportFile[]; token: string; notice: string }>(
      `/downloads/${path(id)}/adoption/preview`,
      { source_root },
    ),
  adopt: (id: string, source_root: string, token: string) =>
    post<{ copied_files: number }>(`/downloads/${path(id)}/adoption`, { source_root, token }),
  discover: (query: string, kind: A.MediaKind, page = 1) =>
    get<{ results: A.DiscoverTitle[]; page: number; total_pages: number; attribution: string }>(
      `/discover?${new URLSearchParams({ query, kind, page: String(page) })}`,
    ),
  title: (kind: A.MediaKind, id: number) =>
    get<{ title: A.DiscoverTitle; episodes: A.CatalogEpisode[] }>(`/titles/${kind}/${id}`),
  requests: () => get<{ requests: A.MediaRequest[] }>("/requests"),
  request: (title: A.DiscoverTitle, scope: A.RequestScope) =>
    post<{ request: A.MediaRequest }>("/requests", {
      kind: title.kind,
      provider_id: title.provider_id,
      scope,
    }),
  decide: (id: string, status: string, note = "") =>
    put<{ request: A.MediaRequest }>(`/requests/${path(id)}`, { status, note }),
  monitored: () => get<{ titles: A.MonitoredTitle[] }>("/monitored"),
  monitor: (id: string, value: A.Monitor) =>
    put<{ monitor: A.Monitor }>(`/monitored/${path(id)}`, {
      enabled: value.enabled,
      scope: value.scope,
      library_id: value.library_id,
      root: value.root,
      profile_id: value.profile_id,
      automatic: value.automatic,
    }),
  releases: (
    id: string,
    options: { season?: number; episode?: number; profile_id?: string; offset?: number },
  ) =>
    post<{ releases: A.Release[]; errors: { indexer_id: string; error: string }[] }>(
      `/titles/${path(id)}/releases`,
      options,
    ),
  downloads: () => get<{ downloads: A.Download[] }>("/downloads"),
  download: (input: A.DownloadInput) => post<{ download: A.Download }>("/downloads", input),
  control: (id: string, action: string, selected_files?: number[]) =>
    put<{ download: A.Download }>(`/downloads/${path(id)}`, { action, selected_files }),
  previewImport: (id: string, body: A.ImportInput) =>
    post<{ files: A.ImportFile[]; token: string; copy_fallback: boolean }>(
      `/downloads/${path(id)}/imports/preview`,
      body,
    ),
  import: (id: string, body: A.ImportInput) =>
    post<{ imports: unknown[] }>(`/downloads/${path(id)}/imports`, body),
  settings: () =>
    get<{ settings: A.AcquisitionConfig; engine: string; network_policy: string }>("/settings"),
  configure: (body: A.AcquisitionConfig) =>
    put<{ settings: A.AcquisitionConfig }>("/settings", body),
  profiles: () => get<{ profiles: A.ProfileRecord[] }>("/profiles"),
  profile: (id: string, profile: A.AcquisitionProfile) =>
    put<A.ProfileRecord>(`/profiles/${path(id)}`, profile),
  indexers: () => get<{ indexers: A.Indexer[] }>("/indexers"),
  indexer: (
    id: string,
    input: Pick<A.Indexer, "name" | "endpoint" | "enabled" | "download_hosts"> & {
      api_key?: string;
    },
  ) => put<{ indexer: A.Indexer }>(`/indexers/${path(id)}`, input),
  testIndexer: (id: string) => post<{ indexer: A.Indexer }>(`/indexers/${path(id)}/test`),
  permissions: (id: string, body: A.AcquisitionPermissions) =>
    put<{ permissions: A.AcquisitionPermissions }>(`/access/${path(id)}`, body),
  events: () => get<{ events: A.AcquisitionEvent[] }>("/events"),
};
