import { request } from "./client";
import type { Item } from "./types";

export type MetadataProvider = "tmdb" | "omdb";
export interface MetadataResult {
  provider: MetadataProvider;
  id: string;
  title: string;
  year: string | number | null;
  kind: "movie" | "tv";
  overview: string | null;
  poster_url: string | null;
}

export interface MetadataFields {
  overview?: string | null;
  year?: number | null;
  end_year?: number | null;
  rating?: number | null;
  content_rating?: string | null;
  genres?: string[];
  studios?: string[];
  studio_images?: { name: string; url: string }[];
  creators?: string[];
  provider_ids?: { tmdb?: string | number; imdb?: string };
  external_urls?: { tmdb?: string; imdb?: string };
  show_overview?: string | null;
  show_year?: number | null;
  show_end_year?: number | null;
  show_content_rating?: string | null;
  show_rating?: number | null;
  show_genres?: string[];
  show_studios?: string[];
  show_creators?: string[];
}

export const metadataApi = {
  search: (id: string, provider: MetadataProvider, query: string) =>
    request<{ results: MetadataResult[] }>(
      `/items/${encodeURIComponent(id)}/metadata/search?${new URLSearchParams({ provider, query })}`,
    ),
  match: (id: string, result: MetadataResult) =>
    request<{ item: Item }>(`/items/${encodeURIComponent(id)}/metadata/match`, {
      method: "POST",
      body: { provider: result.provider, id: result.id },
    }),
};
