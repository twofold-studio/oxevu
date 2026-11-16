export interface AcquisitionPermissions {
  can_request: boolean;
  auto_approve: boolean;
  manage_requests: boolean;
  manage_downloads: boolean;
  manage_indexers: boolean;
  request_limit: number;
}
export type MediaKind = "movie" | "tv";
export interface EpisodeRef {
  season: number;
  episode: number;
}
export interface RequestScope {
  mode: "all" | "existing" | "future" | "seasons" | "episodes" | "requests";
  seasons: number[];
  episodes: EpisodeRef[];
  cutoff: string | null;
}
export interface Availability {
  items: { id: string; season: number | null; episode: number | null }[];
  offline: boolean;
  requested: boolean;
}
export interface DiscoverTitle {
  id: string;
  kind: MediaKind;
  provider_id: number;
  title: string;
  year: string | null;
  overview?: string | null;
  poster_url?: string | null;
  rating?: number | null;
  release_date?: string | null;
  availability?: Availability;
  seasons?: { season_number: number; name: string; episode_count: number }[] | null;
}
export interface CatalogEpisode {
  id: number;
  season_number: number;
  episode_number: number;
  name: string;
  air_date: string | null;
  overview?: string;
  available: boolean;
}
export interface MediaRequest {
  stage: string;
  id: string;
  title: DiscoverTitle;
  user_id: string;
  username: string;
  scope: RequestScope;
  status: "pending" | "approved" | "fulfilled" | "declined" | "cancelled";
  note: string;
  created_at: number;
  availability: Availability;
}
export interface AcquisitionConfig {
  enabled: boolean;
  download_dir: string;
  listen_port: number;
  bind_interface: string | null;
  dht: boolean;
  max_active_downloads: number;
  download_limit_kib: number;
  upload_limit_kib: number;
  min_free_gb: number;
  storage_limit_gb: number;
  seed_ratio: number;
  seed_hours: number;
  search_interval_minutes: number;
  import_enabled: boolean;
  copy_fallback: boolean;
}
export interface AcquisitionProfile {
  name: string;
  min_height: number;
  max_height: number;
  min_size_mb: number;
  max_size_mb: number;
  min_seeders: number;
  codecs: string[];
  sources: string[];
  languages: string[];
  allow_hdr: boolean;
}
export interface ProfileRecord {
  id: string;
  profile: AcquisitionProfile;
}
export interface Indexer {
  id: string;
  name: string;
  endpoint: string;
  enabled: boolean;
  api_key_set: boolean;
  download_hosts: string[];
  capabilities: Record<string, unknown> | null;
}
export interface Release {
  id: string;
  title: string;
  size?: number | null;
  seeders?: number | null;
  height?: number | null;
  codec?: string | null;
  source?: string | null;
  season?: number | null;
  episode?: number | null;
  indexer_id: string;
  indexer_name: string;
  accepted: boolean;
  reasons: string[];
  identity_match: boolean;
  blocked?: boolean;
  infohash?: string;
  also_on?: ReleaseSource[];
}
export interface ReleaseSource {
  indexer_id: string;
  indexer_name: string;
  hits?: number;
}
export interface IndexerError extends ReleaseSource {
  error: string;
  retryable: boolean;
}
export interface Searched {
  indexers: number;
  failed: number;
  took_ms: number;
  hits: number;
  sources: ReleaseSource[];
}
export interface ReleaseResults {
  releases: Release[];
  errors: IndexerError[];
  searched: Searched;
  offset: number;
}
export interface WantedUnit {
  season: number | null;
  episode: number | null;
  name: string;
  air_date: string | null;
  wanted: boolean;
  available: boolean;
  offline: boolean;
  item_id: string | null;
}
export interface Monitor {
  enabled: boolean;
  scope: RequestScope;
  library_id: string | null;
  root: string | null;
  profile_id: string;
  automatic: boolean;
  next_search?: number | null;
  failures?: number | null;
}
export interface MonitoredTitle {
  title: DiscoverTitle;
  units: WantedUnit[];
  monitor: Monitor;
}
export interface TorrentFile {
  index: number;
  path: string;
  size: number;
}
export interface TorrentStats {
  state?: string;
  progress_bytes?: number;
  total_bytes?: number;
  uploaded_bytes?: number;
  finished?: boolean;
  file_progress?: number[];
  live?: {
    download_speed: { mbps: number };
    upload_speed: { mbps: number };
    snapshot?: { peer_stats?: Record<string, number> };
  } | null;
}
export interface Download {
  id: string;
  title_id: string | null;
  title: DiscoverTitle | null;
  infohash: string | null;
  state: string;
  error: string | null;
  stats: TorrentStats;
  files: TorrentFile[];
  selected_files: number[] | null;
  seed_seconds: number;
  created_at: number;
  import_warning?: string | null;
  queue_reason?: string | null;
  release_title?: string | null;
  indexer_name?: string | null;
  release_size?: number | null;
  imported?: boolean;
  import_blocker?: string | null;
  download_limit_kib?: number;
}
export interface DownloadInput {
  paused?: boolean;
  release_id?: string;
  title_id?: string;
  magnet?: string;
  torrent_base64?: string;
  selected_files?: number[];
  override_profile?: boolean;
  profile_id?: string;
  only_wanted?: boolean;
}
export interface ImportMapping {
  file_index: number;
  episodes: EpisodeRef[];
}
export interface ImportInput {
  library_id: string;
  root: string;
  mappings: ImportMapping[];
  token?: string;
}
export interface ImportFile {
  file_index: number;
  source: string;
  destination: string;
  size: number;
  coverage: EpisodeRef[];
}
export interface AcquisitionEvent {
  id: number;
  entity_id: string;
  message: string;
  created_at: number;
}
export interface MigrationInput {
  source: "radarr" | "sonarr" | "seerr";
  records: unknown;
  users: Record<string, string>;
  tmdb_ids: Record<string, number>;
  profile_id?: string;
  token?: string;
}
export interface MigrationPreview {
  token: string;
  count: number;
  notice: string;
  records: {
    key: string;
    title: string;
    kind: string;
    provider_id: number;
    owner: string | null;
    status: string;
    enabled: boolean;
  }[];
}
