import type { MetadataFields } from "./metadata";
import type { AcquisitionPermissions } from "./acquisition-types";

// Mirrors docs/api-phase-one.md and docs/api-playback-settings.md. Field names stay snake_case so values pass
// through unchanged; replace with generated types once the server publishes
// an OpenAPI description.

export type SyncplayAccess = "create_join" | "join" | "disabled";

/** Permission flags the server attaches to the signed-in user. */
export interface UserAccess {
  acquisition?: AcquisitionPermissions;
  is_admin: boolean;
  disabled: boolean;
  can_play: boolean;
  can_transcode: boolean;
  syncplay_access: SyncplayAccess;
}

export interface User extends UserAccess {
  id: string;
  username: string;
}

export interface AuthStatus {
  setup_required: boolean;
  user: User | null;
}

export interface Credentials {
  username: string;
  password: string;
}

/** Server-side rule: passwords must be at least 12 bytes (UTF-8). */
export const MIN_PASSWORD_BYTES = 12;

export type LibraryKind = "movies" | "tv";

export interface Library {
  id: string;
  name: string;
  /** First root, kept for older clients; `paths` is authoritative. */
  path: string;
  paths: string[];
  enabled: boolean;
  kind: LibraryKind;
}

/** Create and update body for `/libraries`. */
export interface LibraryInput {
  name: string;
  kind: LibraryKind;
  paths: string[];
  enabled: boolean;
}

export interface ScanResult {
  scanned: number;
  errors: string[];
}

export type ItemKind = "movie" | "episode";

/** Download/transfer status attached to items backed by an in-progress torrent. */
export interface DownloadStatus {
  id: string;
  file_index: number;
  state: string;
  bytes_received: number;
  size_bytes: number | null;
  playable: boolean;
}

export interface Item extends MetadataFields {
  id: string;
  library_id: string;
  title: string;
  kind: ItemKind;
  show_title: string | null;
  season_number: number | null;
  episode_number: number | null;
  duration_seconds: number | null;
  container: string | null;
  video_codec: string | null;
  audio_codec: string | null;
  size_bytes?: number | null;
  bitrate?: number | null;
  video_width?: number | null;
  video_height?: number | null;
  video_bitrate?: number | null;
  audio_bitrate?: number | null;
  audio_channels?: number | null;
  audio_channel_layout?: string | null;
  position_seconds: number;
  completed: boolean;
  /**
   * Local artwork as relative same-origin URLs
   * (`/api/v1/items/{id}/artwork/{kind}`). Optional because records from
   * servers predating the artwork extension omit them; null when no sidecar
   * image was found. Episodes carry their show's inherited poster.
   */
  poster_url?: string | null;
  thumbnail_url?: string | null;
  backdrop_url?: string | null;
  /** Whether the local media file exists. Missing means true (older servers). */
  file_available?: boolean;
  /** Unix seconds when the file was removed; null if not removed. */
  file_removed_at?: number | null;
  /** Present while the item is backed by an active torrent transfer. */
  download?: DownloadStatus | null;
  /** Per-user opinion: like, dislike, or null (no opinion). */
  reaction?: "like" | "dislike" | null;
}

export type ArtworkKind = "poster" | "thumbnail" | "backdrop";

export interface ProgressUpdate {
  position_seconds: number;
  completed: boolean;
}
