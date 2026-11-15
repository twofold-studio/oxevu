// Hash routing keeps the embedded server simple: it serves one index.html and
// never needs SPA fallbacks for client paths. Routes carry opaque item IDs or
// a show slug; nothing here knows about the catalog, so links are built from
// values the caller already has. Kept free of runes so it can be unit tested.

export type Route =
  | { name: "home" }
  | { name: "movies" }
  | { name: "shows" }
  /** `season` is a `seasonKey`, kept in the URL so Back restores the tab. */
  | { name: "show"; ref: string; season?: string }
  | { name: "item"; id: string }
  | { name: "watch"; id: string; fromStart: boolean }
  | { name: "search"; query: string }
  | { name: "settings"; section: SettingsSection | null }
  | { name: "parties" }
  | { name: "party"; id: string }
  | { name: "not-found" };

export const settingsSections = [
  "libraries",
  "playback",
  "integrations",
  "users",
  "account",
] as const;
export type SettingsSection = (typeof settingsSections)[number];

function isSection(value: string | null): value is SettingsSection {
  return settingsSections.some((section) => section === value);
}

export const href = {
  home: () => "#/",
  movies: () => "#/movies",
  shows: () => "#/shows",
  /** Takes `Show.slug`; legacy key links still resolve through `findShow`. */
  show: (slug: string, season?: string) =>
    `#/shows/${encodeURIComponent(slug)}${season === undefined ? "" : `?season=${encodeURIComponent(season)}`}`,
  item: (id: string) => `#/items/${encodeURIComponent(id)}`,
  watch: (id: string, fromStart = false) =>
    `#/watch/${encodeURIComponent(id)}${fromStart ? "?start=0" : ""}`,
  search: (query: string) => `#/search?q=${encodeURIComponent(query)}`,
  /** Without a section the settings page picks the first one the user may open. */
  settings: (section?: SettingsSection) => `#/settings${section ? `/${section}` : ""}`,
  libraries: () => "#/settings/libraries",
  parties: () => "#/parties",
  party: (id: string) => `#/parties/${encodeURIComponent(id)}`,
};

function decode(part: string): string | null {
  try {
    return decodeURIComponent(part);
  } catch {
    return null;
  }
}

export function parseRoute(hash: string): Route {
  const raw = hash.replace(/^#/, "") || "/";
  const [path, queryString = ""] = raw.split("?", 2);
  const query = new URLSearchParams(queryString);
  const parts = path.split("/").filter(Boolean);
  const param = parts[1] === undefined ? null : decode(parts[1]);

  switch (parts[0]) {
    case undefined:
      return { name: "home" };
    case "movies":
      return parts.length === 1 ? { name: "movies" } : { name: "not-found" };
    case "shows":
      if (parts.length === 1) return { name: "shows" };
      if (!param) return { name: "not-found" };
      return query.has("season")
        ? { name: "show", ref: param, season: query.get("season") ?? "" }
        : { name: "show", ref: param };
    case "items":
      return param ? { name: "item", id: param } : { name: "not-found" };
    case "watch":
      return param
        ? { name: "watch", id: param, fromStart: query.get("start") === "0" }
        : { name: "not-found" };
    case "search":
      return { name: "search", query: query.get("q") ?? "" };
    case "libraries":
      // Links from before the settings area.
      return { name: "settings", section: "libraries" };
    case "settings":
      if (parts.length === 1) return { name: "settings", section: null };
      return parts.length === 2 && isSection(param)
        ? { name: "settings", section: param }
        : { name: "not-found" };
    case "parties":
      if (parts.length === 1) return { name: "parties" };
      return param && parts.length === 2 ? { name: "party", id: param } : { name: "not-found" };
    default:
      return { name: "not-found" };
  }
}
