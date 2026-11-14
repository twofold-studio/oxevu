// Mirrors docs/api-phase-one.md. Field names stay snake_case so values pass
// through unchanged; replace with generated types once the server publishes
// an OpenAPI description.

export interface User {
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
  path: string;
  kind: LibraryKind;
}

export interface NewLibrary {
  name: string;
  path: string;
  kind: LibraryKind;
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
}

export interface ProgressUpdate {
  position_seconds: number;
  completed: boolean;
}
