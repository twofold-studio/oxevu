import { API_BASE } from "./client";
import * as generated from "./generated/sdk.gen";
import { sdkClient, sdkData } from "./sdk";
import type { Credentials, LibraryInput, ProgressUpdate } from "./types";
import type { PasswordChange, Settings, UserInput } from "./admin-types";
import type { PartyControl, PlaybackSessionRequest } from "./playback-types";

const id = (value: string) => encodeURIComponent(value);
const streamRoute: import("./generated/types.gen").MediaStreamData["url"] = "/items/{id}/stream";
const downloadRoute: import("./generated/types.gen").MediaDownloadData["url"] = "/items/{id}/download";
const seekPreviewRoute: import("./generated/types.gen").MediaSeekPreviewData["url"] = "/items/{id}/seek-preview/{seconds}";

export const authApi = {
  status: () => sdkData(generated.status({ client: sdkClient(), throwOnError: true })),
  /**
   * First-admin bootstrap. The one-time token proves local access to the
   * server's data directory (OXEVU_DATA_DIR/setup-token). It is sent once in
   * a header and never stored by the client.
   */
  setup: (body: Credentials, setupToken: string) =>
    sdkData(
      generated.setup({
        client: sdkClient(true),
        throwOnError: true,
        body,
        headers: { "X-Oxevu-Setup-Token": setupToken },
      }),
    ),
  login: (body: Credentials) =>
    sdkData(generated.login({ client: sdkClient(true), throwOnError: true, body })),
  logout: async () => {
    await generated.logout({ client: sdkClient(true), throwOnError: true });
  },
  /** Own account; a wrong current password is reported as 401 or 403, not a lost session. */
  changePassword: (body: PasswordChange) =>
    (async () => {
      await generated.password({ client: sdkClient(true), throwOnError: true, body });
    })(),
};

export const librariesApi = {
  suggestPaths: (path: string) =>
    sdkData(generated.pathsSuggest({ client: sdkClient(), throwOnError: true, body: { path } })),
  list: () => sdkData(generated.list({ client: sdkClient(), throwOnError: true })),
  create: (body: LibraryInput) =>
    sdkData(generated.create({ client: sdkClient(), throwOnError: true, body })),
  update: (libraryId: string, body: LibraryInput) =>
    sdkData(
      generated.update({ client: sdkClient(), throwOnError: true, path: { id: libraryId }, body }),
    ),
  /** Removes catalog configuration only; media files are never touched. */
  remove: async (libraryId: string) => {
    await generated.remove({ client: sdkClient(), throwOnError: true, path: { id: libraryId } });
  },
  scan: (libraryId: string) =>
    sdkData(generated.scan({ client: sdkClient(), throwOnError: true, path: { id: libraryId } })),
};

// --- Media removal types ---------------------------------------------------

export type RemovalPreviewItem = import("./generated/types.gen").RemovalPreviewItem;

export type RemovalPreview = import("./generated/types.gen").RemovalPreview;

export type RemovalResult = import("./generated/types.gen").RemovalResult;

export const removalApi = {
  preview: (item_ids: string[], keep_metadata: boolean) =>
    sdkData(
      generated.removalPreview({
        client: sdkClient(),
        throwOnError: true,
        body: { item_ids, keep_metadata },
      }),
    ),
  remove: (item_ids: string[], keep_metadata: boolean, token: string) =>
    sdkData(
      generated.removalApply({
        client: sdkClient(),
        throwOnError: true,
        body: { item_ids, keep_metadata, token, confirm: true },
      }),
    ),
};

export const itemsApi = {
  favorite: async (itemId: string, favorite: boolean) => {
    await generated.favorite({
      client: sdkClient(),
      throwOnError: true,
      path: { id: itemId },
      body: { favorite },
    });
  },
  list: (signal?: AbortSignal) =>
    sdkData(generated.items({ client: sdkClient(), throwOnError: true, signal })),
  get: (itemId: string, signal?: AbortSignal) =>
    sdkData(
      generated.item({ client: sdkClient(), throwOnError: true, path: { id: itemId }, signal }),
    ),
  saveProgress: async (itemId: string, body: ProgressUpdate, keepalive = false) => {
    await generated.progress({
      client: sdkClient(),
      throwOnError: true,
      path: { id: itemId },
      body,
      keepalive,
    });
  },
  /** Byte-range original file. The session cookie authorizes the <video> request. */
  streamUrl: (itemId: string) => `${API_BASE}${streamRoute.replace("{id}", id(itemId))}`,
  downloadUrl: (itemId: string) => `${API_BASE}${downloadRoute.replace("{id}", id(itemId))}`,
  seekPreviewUrl: (itemId: string, seconds: number) =>
    `${API_BASE}${seekPreviewRoute.replace("{id}", id(itemId)).replace("{seconds}", String(seconds))}`,
  reaction: (itemId: string, value: "like" | "dislike" | null) =>
    sdkData(
      generated.reactionUpdate({
        client: sdkClient(),
        throwOnError: true,
        path: { id: itemId },
        body: { value },
      }),
    ),
};

export const usersApi = {
  avatar: (image: string) =>
    sdkData(generated.avatarSave({ client: sdkClient(), throwOnError: true, body: { image } })),
  list: () => sdkData(generated.usersList({ client: sdkClient(), throwOnError: true })),
  create: (body: UserInput) =>
    sdkData(generated.usersCreate({ client: sdkClient(), throwOnError: true, body })),
  update: (userId: string, body: UserInput) =>
    sdkData(
      generated.usersUpdate({
        client: sdkClient(),
        throwOnError: true,
        path: { id: userId },
        body,
      }),
    ),
};

export const settingsApi = {
  get: () => sdkData(generated.settingsGet({ client: sdkClient(), throwOnError: true })),
  save: (body: Settings) =>
    sdkData(generated.settingsPut({ client: sdkClient(), throwOnError: true, body })),
};

export const playbackApi = {
  capabilities: () =>
    sdkData(generated.playbackCapabilities({ client: sdkClient(), throwOnError: true })),
  create: (body: PlaybackSessionRequest) =>
    sdkData(generated.playbackCreate({ client: sdkClient(), throwOnError: true, body })),
  get: (sessionId: string) =>
    sdkData(
      generated.playbackGet({ client: sdkClient(), throwOnError: true, path: { id: sessionId } }),
    ),
  heartbeat: async (sessionId: string) => {
    await generated.playbackHeartbeat({
      client: sdkClient(),
      throwOnError: true,
      path: { id: sessionId },
    });
  },
  /** keepalive lets the stop reach the server while the page unloads. */
  stop: async (sessionId: string, keepalive = false) => {
    await generated.playbackStop({
      client: sdkClient(),
      throwOnError: true,
      path: { id: sessionId },
      keepalive,
    });
  },
};

export const partiesApi = {
  list: () => sdkData(generated.partyList({ client: sdkClient(), throwOnError: true })),
  create: (body: { name: string; item_id?: string | null }) =>
    sdkData(generated.partyCreate({ client: sdkClient(), throwOnError: true, body })),
  get: (partyId: string) =>
    sdkData(
      generated.partyRead({ client: sdkClient(), throwOnError: true, path: { id: partyId } }),
    ),
  join: (partyId: string) =>
    sdkData(
      generated.partyJoin({ client: sdkClient(), throwOnError: true, path: { id: partyId } }),
    ),
  leave: async (partyId: string, keepalive = false) => {
    await generated.partyLeave({
      client: sdkClient(),
      throwOnError: true,
      path: { id: partyId },
      keepalive,
    });
  },
  control: (partyId: string, body: PartyControl) =>
    sdkData(
      generated.partyControl({
        client: sdkClient(),
        throwOnError: true,
        path: { id: partyId },
        body,
      }),
    ),
};

export const subtitlesApi = {
  list: (id: string) =>
    sdkData(generated.subtitlesList({ client: sdkClient(), throwOnError: true, path: { id } })),
  search: (id: string, language: string) =>
    sdkData(
      generated.subtitlesSearch({
        client: sdkClient(),
        throwOnError: true,
        path: { id },
        query: { language },
      }),
    ),
  download: (id: string, subtitleId: string, language: string) =>
    sdkData(
      generated.subtitlesDownload({
        client: sdkClient(),
        throwOnError: true,
        path: { id },
        body: { id: subtitleId, language },
      }),
    ),
};
