// Hash routing keeps the embedded server simple: it serves one index.html and
// never needs SPA fallbacks for client paths. Routes carry opaque item IDs or
// a show slug; nothing here knows about the catalog, so links are built from
// values the caller already has. Kept free of runes so it can be unit tested.

import {
  isSettingsAlias,
  isSettingsPage,
  type SettingsAlias,
  type SettingsPageId,
  type SettingsTarget,
} from "./settings-pages";

export type Route =
  | { name: "home" }
  | { name: "movies" }
  | { name: "shows" }
  /** `season` is a `seasonKey`, kept in the URL so Back restores the tab. */
  | { name: "show"; ref: string; season?: string }
  | { name: "item"; id: string }
  | { name: "watch"; id: string; fromStart: boolean }
  | { name: "search"; query: string }
  /** `target` is a page, an alias that opens the first permitted page, or the index. */
  | { name: "settings"; target: SettingsTarget }
  | { name: "parties" }
  | { name: "party"; id: string }
  | { name: "discover"; query: string; kind: "movie" | "tv"; page: number }
  | { name: "discover-title"; kind: "movie" | "tv"; id: number }
  | { name: "requests" }
  | { name: "wanted" }
  | { name: "activity" }
  | { name: "not-found" };

/** Parses `#/settings/<group>/<page>` and the shorter aliases in settings-pages. */
function parseSettings(parts: string[]): Route {
  if (parts.length === 1) return { name: "settings", target: null };
  const decoded = parts.slice(1).map(decode);
  if (decoded.some((part) => part === null)) return { name: "not-found" };
  const path = decoded.join("/");
  if (parts.length === 2 && isSettingsAlias(path)) return { name: "settings", target: path };
  if (parts.length === 3 && isSettingsPage(path)) return { name: "settings", target: path };
  return { name: "not-found" };
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
  /** Without a page the settings area picks the first one the user may open. */
  settings: (target?: SettingsPageId | SettingsAlias) => `#/settings${target ? `/${target}` : ""}`,
  libraries: () => "#/settings/media/libraries",
  parties: () => "#/parties",
  party: (id: string) => `#/parties/${encodeURIComponent(id)}`,
  discover: (query = "", kind: "movie" | "tv" = "movie", page = 1) =>
    `#/discover?${new URLSearchParams({ q: query, kind, page: String(page) })}`,
  discoverTitle: (kind: "movie" | "tv", id: number) => `#/discover/${kind}/${id}`,
  requests: () => "#/requests",
  wanted: () => "#/wanted",
  activity: () => "#/activity",
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
      return parts.length === 1 ? { name: "settings", target: "libraries" } : { name: "not-found" };
    case "settings":
      return parseSettings(parts);
    case "parties":
      if (parts.length === 1) return { name: "parties" };
      return param && parts.length === 2 ? { name: "party", id: param } : { name: "not-found" };
    case "discover": {
      if (parts.length === 1)
        return {
          name: "discover",
          query: query.get("q") ?? "",
          kind: query.get("kind") === "tv" ? "tv" : "movie",
          page: Math.min(20, Math.max(1, Number(query.get("page")) || 1)),
        };
      const id = Number(parts[2]);
      return parts.length === 3 &&
        (param === "movie" || param === "tv") &&
        Number.isSafeInteger(id) &&
        id > 0
        ? { name: "discover-title", kind: param, id }
        : { name: "not-found" };
    }
    case "requests":
      return parts.length === 1 ? { name: "requests" } : { name: "not-found" };
    case "wanted":
      return parts.length === 1 ? { name: "wanted" } : { name: "not-found" };
    case "activity":
      return parts.length === 1 ? { name: "activity" } : { name: "not-found" };
    default:
      return { name: "not-found" };
  }
}
