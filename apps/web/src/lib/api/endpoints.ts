import { API_BASE, request } from "./client";
import type {
  AuthStatus,
  Credentials,
  Item,
  Library,
  LibraryInput,
  ProgressUpdate,
  ScanResult,
  User,
} from "./types";
import type {
  Capabilities,
  PasswordChange,
  Settings,
  SettingsEnvelope,
  UserInput,
  UserSettings,
} from "./admin-types";
import type {
  Party,
  PartyControl,
  PlaybackSession,
  PlaybackSessionRequest,
} from "./playback-types";

const id = (value: string) => encodeURIComponent(value);

export const authApi = {
  status: () => request<AuthStatus>("/auth/status"),
  /**
   * First-admin bootstrap. The one-time token proves local access to the
   * server's data directory (OXEVU_DATA_DIR/setup-token). It is sent once in
   * a header and never stored by the client.
   */
  setup: (body: Credentials, setupToken: string) =>
    request<{ user: User }>("/auth/setup", {
      method: "POST",
      body,
      ignoreUnauthorized: true,
      headers: { "X-Oxevu-Setup-Token": setupToken },
    }),
  login: (body: Credentials) =>
    request<{ user: User }>("/auth/login", { method: "POST", body, ignoreUnauthorized: true }),
  logout: () => request<void>("/auth/logout", { method: "POST", ignoreUnauthorized: true }),
  /** Own account; a wrong current password is reported as 401 or 403, not a lost session. */
  changePassword: (body: PasswordChange) =>
    request<void>("/auth/password", { method: "PUT", body, ignoreUnauthorized: true }),
};

export const librariesApi = {
  suggestPaths: (path: string) =>
    request<{ paths: { name: string; path: string }[]; truncated: boolean }>(
      "/libraries/paths/suggest",
      { method: "POST", body: { path } },
    ),
  list: () => request<{ libraries: Library[] }>("/libraries"),
  create: (body: LibraryInput) =>
    request<{ library: Library }>("/libraries", { method: "POST", body }),
  update: (libraryId: string, body: LibraryInput) =>
    request<{ library: Library }>(`/libraries/${id(libraryId)}`, { method: "PUT", body }),
  /** Removes catalog configuration only; media files are never touched. */
  remove: (libraryId: string) =>
    request<void>(`/libraries/${id(libraryId)}/remove`, { method: "POST" }),
  scan: (libraryId: string) =>
    request<ScanResult>(`/libraries/${id(libraryId)}/scan`, { method: "POST" }),
};

// --- Media removal types ---------------------------------------------------

export interface RemovalPreviewItem {
  id: string;
  title: string;
  size_bytes: number | null;
  local_file: boolean;
  shared_links: number;
}

export interface RemovalPreview {
  token: string;
  count: number;
  keep_metadata: boolean;
  bytes_selected: number;
  shared_files_count: number;
  items: RemovalPreviewItem[];
  notice: string;
}

export interface RemovalResult {
  deleted_files?: number;
  removed_ids: string[];
  kept_metadata: boolean;
  failures: { id: string; message: string }[];
  bytes_freed_estimate: number;
  shared_files_count: number;
}

export const removalApi = {
  preview: (item_ids: string[], keep_metadata: boolean) =>
    request<RemovalPreview>("/library/removals/preview", {
      method: "POST",
      body: { item_ids, keep_metadata },
    }),
  remove: (item_ids: string[], keep_metadata: boolean, token: string) =>
    request<RemovalResult>("/library/removals", {
      method: "POST",
      body: { item_ids, keep_metadata, token, confirm: true },
    }),
};

export const itemsApi = {
  favorite: (itemId: string, favorite: boolean) =>
    request<void>(`/items/${id(itemId)}/favorite`, { method: "PUT", body: { favorite } }),
  list: (signal?: AbortSignal) => request<{ items: Item[] }>("/items", { signal }),
  get: (itemId: string, signal?: AbortSignal) =>
    request<{ item: Item }>(`/items/${id(itemId)}`, { signal }),
  saveProgress: (itemId: string, body: ProgressUpdate, keepalive = false) =>
    request<void>(`/items/${id(itemId)}/progress`, { method: "PUT", body, keepalive }),
  /** Byte-range original file. The session cookie authorizes the <video> request. */
  streamUrl: (itemId: string) => `${API_BASE}/items/${id(itemId)}/stream`,
  reaction: (itemId: string, value: "like" | "dislike" | null) =>
    request<{ value: "like" | "dislike" | null }>(`/items/${id(itemId)}/reaction`, {
      method: "PUT",
      body: { value },
    }),
};

export const usersApi = {
  list: () => request<{ users: UserSettings[] }>("/users"),
  create: (body: UserInput) => request<{ user: UserSettings }>("/users", { method: "POST", body }),
  update: (userId: string, body: UserInput) =>
    request<{ user: UserSettings }>(`/users/${id(userId)}`, { method: "PUT", body }),
};

export const settingsApi = {
  get: () => request<SettingsEnvelope>("/settings"),
  save: (body: Settings) => request<SettingsEnvelope>("/settings", { method: "PUT", body }),
};

export const playbackApi = {
  capabilities: () => request<{ capabilities: Capabilities }>("/playback/capabilities"),
  create: (body: PlaybackSessionRequest) =>
    request<{ session: PlaybackSession }>("/playback/sessions", { method: "POST", body }),
  get: (sessionId: string) =>
    request<{ session: PlaybackSession }>(`/playback/sessions/${id(sessionId)}`),
  heartbeat: (sessionId: string) =>
    request<void>(`/playback/sessions/${id(sessionId)}/heartbeat`, { method: "POST" }),
  /** keepalive lets the stop reach the server while the page unloads. */
  stop: (sessionId: string, keepalive = false) =>
    request<void>(`/playback/sessions/${id(sessionId)}/stop`, { method: "POST", keepalive }),
};

export const partiesApi = {
  list: () => request<{ parties: Party[] }>("/parties"),
  create: (body: { name: string; item_id?: string | null }) =>
    request<{ party: Party }>("/parties", { method: "POST", body }),
  get: (partyId: string) => request<{ party: Party }>(`/parties/${id(partyId)}`),
  join: (partyId: string) =>
    request<{ party: Party }>(`/parties/${id(partyId)}/join`, { method: "POST" }),
  leave: (partyId: string, keepalive = false) =>
    request<void>(`/parties/${id(partyId)}/leave`, { method: "POST", keepalive }),
  control: (partyId: string, body: PartyControl) =>
    request<{ party: Party }>(`/parties/${id(partyId)}/control`, { method: "POST", body }),
};
