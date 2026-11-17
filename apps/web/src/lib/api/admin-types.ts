// Administration resources from docs/api-playback-settings.md.
import type { SyncplayAccess, UserAccess } from "./types";

/** A user as administrators see and edit it. */
export type UserSettings = import("./generated/types.gen").User;

type WireInput = import("./generated/types.gen").UserInput;
/** The editor fills all permission flags; the REST input also accepts defaults. */
export type UserInput = {
  [
    K in
      | "username"
      | "is_admin"
      | "disabled"
      | "can_play"
      | "can_transcode"
      | "all_libraries"
      | "syncplay_access"
      | "library_ids"
  ]: NonNullable<WireInput[K]>;
} & Pick<WireInput, "password"> &
  Pick<UserAccess, "acquisition">;

export type { SyncplayAccess };

export type HardwareAcceleration = import("./generated/types.gen").HardwareAcceleration;

export type Settings = import("./generated/types.gen").Settings;

/** Runtime probe results; paths come from server environment, not settings. */
export type Capabilities = import("./generated/types.gen").Capabilities;

export type SettingsEnvelope = import("./generated/types.gen").SettingsEnvelope;

export type PasswordChange = import("./generated/types.gen").PasswordChange;
