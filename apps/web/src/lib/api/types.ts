import type * as Wire from "./generated/types.gen";

// Wire types come from Rust's generated OpenAPI contract. Local form/view
// models remain separate where they intentionally accept only a subset.

export type SyncplayAccess = import("./generated/types.gen").SyncplayAccess;

/** Permission flags the server attaches to the signed-in user. */
export type UserAccess = Pick<
  Wire.User,
  "is_admin" | "disabled" | "can_play" | "can_transcode" | "syncplay_access"
> &
  Partial<Pick<Wire.User, "acquisition">>;

export type User = UserAccess &
  Pick<Wire.User, "id" | "username"> &
  Partial<Pick<Wire.User, "avatar">>;

export type AuthStatus = import("./generated/types.gen").AuthStatus;

export type Credentials = import("./generated/types.gen").Credentials;

/** Server-side rule: passwords must be at least 12 bytes (UTF-8). */
export const MIN_PASSWORD_BYTES = 12;

export type LibraryKind = Wire.LibraryKind;

export type Library = import("./generated/types.gen").Library;

/** Create and update body for `/libraries`. */
export type LibraryInput = {
  [K in "name" | "kind" | "paths" | "enabled"]: NonNullable<Wire.LibraryInput[K]>;
};

export type ScanResult = import("./generated/types.gen").ScanResult;

export type ItemKind = Wire.ItemKind;

/** Download/transfer status attached to items backed by an in-progress torrent. */
export type DownloadStatus = Wire.DownloadStatus;

export type Item = import("./generated/types.gen").Item;

export type ArtworkKind = "poster" | "thumbnail" | "backdrop";

export type ProgressUpdate = import("./generated/types.gen").ProgressUpdate;
