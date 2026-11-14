import { API_BASE, request } from "./client";
import type {
  AuthStatus,
  Credentials,
  Item,
  Library,
  NewLibrary,
  ProgressUpdate,
  ScanResult,
  User,
} from "./types";

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
};

export const librariesApi = {
  list: () => request<{ libraries: Library[] }>("/libraries"),
  create: (body: NewLibrary) =>
    request<{ library: Library }>("/libraries", { method: "POST", body }),
  scan: (libraryId: string) =>
    request<ScanResult>(`/libraries/${id(libraryId)}/scan`, { method: "POST" }),
};

export const itemsApi = {
  list: () => request<{ items: Item[] }>("/items"),
  get: (itemId: string) => request<{ item: Item }>(`/items/${id(itemId)}`),
  saveProgress: (itemId: string, body: ProgressUpdate, keepalive = false) =>
    request<void>(`/items/${id(itemId)}/progress`, { method: "PUT", body, keepalive }),
  /** Byte-range original file. The session cookie authorizes the <video> request. */
  streamUrl: (itemId: string) => `${API_BASE}/items/${id(itemId)}/stream`,
};
