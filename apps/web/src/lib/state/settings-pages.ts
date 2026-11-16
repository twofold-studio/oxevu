// Settings destinations, their URLs and who may open them. Kept free of runes
// and icons so routing and access rules can be unit tested. The server still
// authorizes every request; this only decides what navigation offers.

import type { UserAccess } from "$lib/api/types";

export const settingsGroups = [
  {
    id: "media",
    label: "Media",
    pages: [
      { id: "media/libraries", label: "Libraries" },
      { id: "media/metadata", label: "Metadata" },
      { id: "media/subtitles", label: "Subtitles" },
    ],
  },
  {
    id: "playback",
    label: "Playback",
    pages: [
      { id: "playback/conversion", label: "Conversion" },
      { id: "playback/quality", label: "Quality limits" },
      { id: "playback/scratch", label: "Scratch and cache" },
      { id: "playback/resume", label: "Resume" },
    ],
  },
  {
    id: "downloads",
    label: "Downloads",
    pages: [
      { id: "downloads/storage", label: "Engine and storage" },
      { id: "downloads/network", label: "Network" },
      { id: "downloads/queue", label: "Queue and search" },
      { id: "downloads/seeding", label: "Seeding" },
      { id: "downloads/imports", label: "Library imports" },
    ],
  },
  {
    id: "sources",
    label: "Sources",
    pages: [
      { id: "sources/indexers", label: "Indexers" },
      { id: "sources/profiles", label: "Release profiles" },
      { id: "sources/migration", label: "Migration" },
    ],
  },
  {
    id: "people",
    label: "People",
    pages: [
      { id: "people/users", label: "Users" },
      { id: "people/requests", label: "Request access" },
    ],
  },
  {
    id: "account",
    label: "Your account",
    pages: [
      { id: "account/profile", label: "Profile" },
      { id: "account/password", label: "Password" },
      { id: "account/player", label: "Player" },
    ],
  },
] as const;

type Group = (typeof settingsGroups)[number];
export type SettingsGroupId = Group["id"];
export type SettingsPageId = Group["pages"][number]["id"];

const pages: { id: SettingsPageId; label: string; group: Group }[] = settingsGroups.flatMap(
  (group) => group.pages.map((page) => ({ id: page.id, label: page.label, group })),
);
export const settingsPageIds: SettingsPageId[] = pages.map((page) => page.id);

const groupPages = (...ids: SettingsGroupId[]): SettingsPageId[] =>
  pages.filter((page) => ids.includes(page.group.id)).map((page) => page.id);

/**
 * Shorter addresses that open the first permitted page they cover. Includes
 * every tab URL from before the settings navigation existed, so old bookmarks
 * and documentation links keep working.
 */
export const settingsAliases = {
  media: groupPages("media"),
  playback: groupPages("playback"),
  downloads: groupPages("downloads"),
  sources: groupPages("sources"),
  people: groupPages("people"),
  account: groupPages("account"),
  // Legacy tabs.
  libraries: ["media/libraries"],
  integrations: ["media/metadata", "media/subtitles"],
  acquisition: groupPages("downloads", "sources"),
  users: ["people/users", "people/requests"],
} satisfies Record<string, SettingsPageId[]>;
export type SettingsAlias = keyof typeof settingsAliases;

/** What a settings URL asks for: a page, an alias, or nothing (the index). */
export type SettingsTarget = SettingsPageId | SettingsAlias | null;

export const isSettingsPage = (value: string): value is SettingsPageId =>
  settingsPageIds.some((id) => id === value);
export const isSettingsAlias = (value: string): value is SettingsAlias =>
  Object.hasOwn(settingsAliases, value);

export function settingsPageLabel(id: SettingsPageId): { page: string; group: string } {
  const page = pages.find((entry) => entry.id === id);
  return { page: page?.label ?? id, group: page?.group.label ?? "" };
}

type Access = Partial<UserAccess> | null | undefined;

/** Who may open each page. Delegated indexer managers get indexers only. */
export function canOpenSettingsPage(user: Access, page: SettingsPageId): boolean {
  if (page.startsWith("account/")) return true;
  if (user?.is_admin === true) return true;
  return page === "sources/indexers" && user?.acquisition?.manage_indexers === true;
}

/** Groups and pages the user may open, in navigation order; empty groups are dropped. */
export function settingsNavFor(user: Access) {
  return settingsGroups
    .map((group) => ({
      id: group.id,
      label: group.label,
      pages: group.pages.filter((page) => canOpenSettingsPage(user, page.id)),
    }))
    .filter((group) => group.pages.length > 0);
}

export type SettingsResolution =
  /** Show the page. */
  | { kind: "page"; page: SettingsPageId }
  /** Replace the URL with this page; an alias or the bare index was opened. */
  | { kind: "redirect"; page: SettingsPageId }
  /** The bare index: phones list destinations, wider screens redirect to `first`. */
  | { kind: "index"; first: SettingsPageId }
  /** Opened by URL without permission: explain instead of redirecting elsewhere. */
  | { kind: "forbidden"; page: SettingsPageId };

export function resolveSettings(target: SettingsTarget, user: Access): SettingsResolution {
  if (target === null) {
    // Account pages are open to everyone, so there is always a first page.
    const first = settingsPageIds.find((id) => canOpenSettingsPage(user, id)) ?? "account/profile";
    return { kind: "index", first };
  }
  if (isSettingsPage(target))
    return canOpenSettingsPage(user, target)
      ? { kind: "page", page: target }
      : { kind: "forbidden", page: target };
  const candidates: SettingsPageId[] = settingsAliases[target];
  const allowed = candidates.find((id) => canOpenSettingsPage(user, id));
  return allowed ? { kind: "redirect", page: allowed } : { kind: "forbidden", page: candidates[0] };
}
