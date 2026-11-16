import type { UserAccess } from "$lib/api/types";

// Navigation hints only; the server authorizes every request and the pages
// still explain a 403 when a role changed after sign-in.

type Access = Partial<UserAccess> | null | undefined;

export const isAdmin = (user: Access) => user?.is_admin === true;

export const canJoinParties = (user: Access) =>
  user?.syncplay_access === "join" || user?.syncplay_access === "create_join";

export const canCreateParties = (user: Access) => user?.syncplay_access === "create_join";

// Settings page permissions live beside the page catalog.
export { canOpenSettingsPage, settingsNavFor } from "./settings-pages";
