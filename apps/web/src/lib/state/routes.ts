// Hash routing keeps the embedded server simple: it serves one index.html and
// never needs SPA fallbacks for client paths. Routes carry opaque item IDs or
// a show slug; nothing here knows about the catalog, so links are built from
// values the caller already has. Kept free of runes so it can be unit tested.

export type Route =
  | { name: "home" }
  | { name: "movies" }
  | { name: "shows" }
  | { name: "show"; ref: string }
  | { name: "item"; id: string }
  | { name: "watch"; id: string; fromStart: boolean }
  | { name: "search"; query: string }
  | { name: "libraries" }
  | { name: "not-found" };

export const href = {
  home: () => "#/",
  movies: () => "#/movies",
  shows: () => "#/shows",
  /** Takes `Show.slug`; legacy key links still resolve through `findShow`. */
  show: (slug: string) => `#/shows/${encodeURIComponent(slug)}`,
  item: (id: string) => `#/items/${encodeURIComponent(id)}`,
  watch: (id: string, fromStart = false) =>
    `#/watch/${encodeURIComponent(id)}${fromStart ? "?start=0" : ""}`,
  search: (query: string) => `#/search?q=${encodeURIComponent(query)}`,
  libraries: () => "#/libraries",
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
      return param ? { name: "show", ref: param } : { name: "not-found" };
    case "items":
      return param ? { name: "item", id: param } : { name: "not-found" };
    case "watch":
      return param
        ? { name: "watch", id: param, fromStart: query.get("start") === "0" }
        : { name: "not-found" };
    case "search":
      return { name: "search", query: query.get("q") ?? "" };
    case "libraries":
      return { name: "libraries" };
    default:
      return { name: "not-found" };
  }
}
