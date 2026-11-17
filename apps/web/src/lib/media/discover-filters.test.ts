import { describe, expect, test } from "vite-plus/test";
import { applicableFilters, discoverQuery, readDiscoverFilters } from "./discover-filters";

describe("Discover filter URLs", () => {
  test("preserves browse filters through pagination and links", () => {
    const filters = {
      year: 2024,
      genre: 18,
      min_rating: 7,
      language: "bg",
      sort: "rating" as const,
    };
    const params = new URLSearchParams(discoverQuery("", "tv", 3, filters));
    expect(params.get("page")).toBe("3");
    expect(readDiscoverFilters(params)).toEqual(filters);
  });
  test("title search retains year and drops unsupported browse filters", () => {
    expect(applicableFilters("House", { year: 2004, genre: 18, sort: "popular" })).toEqual({
      year: 2004,
    });
    expect(new URLSearchParams(discoverQuery("House", "tv", 1, { genre: 18 })).has("genre")).toBe(
      false,
    );
  });
  test("ignores invalid URL values", () => {
    expect(
      readDiscoverFilters(
        new URLSearchParams("year=0&genre=-1&min_rating=11&language=english&sort=unsafe"),
      ),
    ).toEqual({});
  });
});
