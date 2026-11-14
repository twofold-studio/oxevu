// Mirrors docs/api-phase-one.md and docs/api-playback-settings.md. Field names stay snake_case so values pass
// through unchanged; replace with generated types once the server publishes
// an OpenAPI description.

export type SyncplayAccess = "create_join" | "join" | "disabled";

/** Permission flags the server attaches to the signed-in user. */
export interface UserAccess {
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

export interface Item {
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
}

export type ArtworkKind = "poster" | "thumbnail" | "backdrop";

export interface ProgressUpdate {
  position_seconds: number;
  completed: boolean;
}
