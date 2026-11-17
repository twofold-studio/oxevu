import type { DiscoverData } from "$lib/api/generated/types.gen";

export type DiscoverFilters = Pick<
  NonNullable<DiscoverData["query"]>,
  "year" | "genre" | "min_rating" | "language" | "sort"
>;

export function readDiscoverFilters(params: URLSearchParams): DiscoverFilters {
  const filters: DiscoverFilters = {};
  for (const key of ["year", "genre", "min_rating"] as const) {
    const raw = params.get(key);
    if (!raw) continue;
    const value = Number(raw);
    if (!Number.isFinite(value)) continue;
    if (key === "year" && Number.isInteger(value) && value >= 1880 && value <= 2100)
      filters.year = value;
    if (key === "genre" && Number.isInteger(value) && value > 0 && value <= 1000000)
      filters.genre = value;
    if (key === "min_rating" && value >= 0 && value <= 10) filters.min_rating = value;
  }
  const language = params.get("language");
  if (language && /^[a-z]{2}$/.test(language)) filters.language = language;
  const sort = params.get("sort");
  if (sort === "popular" || sort === "rating" || sort === "newest" || sort === "oldest")
    filters.sort = sort;
  return filters;
}

/** TMDB title search supports year; its other filters belong to browsing. */
export function applicableFilters(query: string, filters: DiscoverFilters): DiscoverFilters {
  return query.trim() ? (filters.year === undefined ? {} : { year: filters.year }) : filters;
}

export function discoverQuery(
  query: string,
  kind: "movie" | "tv",
  page: number,
  filters: DiscoverFilters,
): string {
  const params = new URLSearchParams({ q: query, kind, page: String(page) });
  for (const [key, value] of Object.entries(applicableFilters(query, filters))) {
    if (value !== undefined) params.set(key, String(value));
  }
  return params.toString();
}
