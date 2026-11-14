import type { UserAccess } from "$lib/api/types";
import type { SettingsSection } from "./routes";

// Navigation hints only; the server authorizes every request and the pages
// still explain a 403 when a role changed after sign-in.

type Access = Partial<UserAccess> | null | undefined;

export const isAdmin = (user: Access) => user?.is_admin === true;

export const canJoinParties = (user: Access) =>
  user?.syncplay_access === "join" || user?.syncplay_access === "create_join";

export const canCreateParties = (user: Access) => user?.syncplay_access === "create_join";

/** Settings sections a user may open, in tab order. */
export function settingsSectionsFor(user: Access): SettingsSection[] {
  return isAdmin(user) ? ["libraries", "playback", "users", "account"] : ["account"];
}
