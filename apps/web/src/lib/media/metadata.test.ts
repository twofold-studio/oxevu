import { describe, expect, test } from "vite-plus/test";
import type { Item } from "$lib/api/types";
import {
  endsAt,
  identificationQuery,
  itemRating,
  present,
  ratingLabel,
  ratingSource,
  seasonRating,
  validScore,
  yearSpan,
} from "./metadata";

test("identification search keeps years that are part of a movie title", () => {
  expect(identificationQuery(item({ title: "Blade Runner 2049 2017 1080p BluRay" }))).toBe("Blade Runner 2049");
  expect(identificationQuery(item({ title: "2001 A Space Odyssey (1968) 1080p" }))).toBe("2001 A Space Odyssey");
});

function item(extra: Partial<Item> = {}): Item {
  return {
    id: "a",
    library_id: "l",
    title: "Title",
    kind: "movie",
    show_title: null,
    season_number: null,
    episode_number: null,
    duration_seconds: null,
    container: null,
    video_codec: null,
    audio_codec: null,
    position_seconds: 0,
    completed: false,
    ...extra,
  };
}

describe("ratings", () => {
  test("only a stored source attributes a score", () => {
    expect(ratingSource("TMDB")).toEqual({ id: "tmdb", name: "TMDB" });
    expect(ratingSource("imdb")).toEqual({ id: "imdb", name: "IMDb" });
    expect(ratingSource("Letterboxd")).toEqual({ id: null, name: "Letterboxd" });
    expect(ratingSource(" ")).toBeNull();
    // Provider ids alone never imply where the score came from.
    expect(itemRating(item({ rating: 7.1, provider_ids: { tmdb: 1 } }))?.source).toBeNull();
  });

  test("drops missing, zero and out-of-range scores", () => {
    expect(validScore(0)).toBeNull();
    expect(validScore(11)).toBeNull();
    expect(validScore(Number.NaN)).toBeNull();
    expect(validScore(8.4)).toBe(8.4);
  });

  test("episodes need an attributed score of their own", () => {
    const copied = item({ kind: "episode", rating: 8, show_rating: 8 });
    expect(itemRating(copied)).toBeNull();
    const own = item({ kind: "episode", rating: 7.5, rating_source: "TMDB" });
    expect(itemRating(own)).toEqual({ value: 7.5, source: { id: "tmdb", name: "TMDB" } });
  });

  test("series and season scores are scoped", () => {
    const show = item({ kind: "episode", show_rating: 8.2, season_rating: 7.9 });
    expect(ratingLabel(itemRating(show, true)!)).toBe("Series rating 8.2 out of 10");
    expect(ratingLabel(seasonRating(show)!)).toBe("TMDB season rating 7.9 out of 10");
    expect(seasonRating(item())).toBeNull();
  });
});

describe("years", () => {
  test("movies use their release year", () => {
    expect(yearSpan(item({ year: 2021, end_year: 2023 }))).toEqual({
      text: "2021",
      label: "Released 2021",
    });
  });

  test("episodes and series show the series span", () => {
    const episode = item({ kind: "episode", year: 2019, show_year: 2019, show_end_year: 2023 });
    expect(yearSpan(episode)?.text).toBe("2019–2023");
    expect(yearSpan(episode, true)?.label).toBe("Series aired 2019 to 2023");
    expect(yearSpan(item({ kind: "episode", show_year: 2020, show_end_year: 2020 }))?.text).toBe(
      "2020",
    );
    expect(yearSpan(item())).toBeNull();
  });
});

describe("text and time", () => {
  test("hides provider placeholders", () => {
    expect(present("N/A")).toBeNull();
    expect(present(" PG-13 ")).toBe("PG-13");
  });

  test("finish time accounts for the resume point", () => {
    const now = new Date(2026, 0, 1, 20, 0).getTime();
    expect(endsAt(3600, 1800, now, "en-GB")).toBe("20:30");
    expect(endsAt(null, 0, now)).toBeNull();
  });
});
