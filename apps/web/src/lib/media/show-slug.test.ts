import { describe, expect, test } from "vite-plus/test";
import type { Item } from "$lib/api/types";
import { groupShows } from "./grouping";
import { assignShowSlugs, findShow, slugify } from "./show-slug";

const LIB_A = "11111111-1111-4111-8111-111111111111";
const LIB_B = "22222222-2222-4222-8222-222222222222";

let nextId = 0;
function episode(libraryId: string, showTitle: string): Item {
  nextId += 1;
  return {
    id: `item-${nextId}`,
    library_id: libraryId,
    title: `Episode ${nextId}`,
    kind: "episode",
    show_title: showTitle,
    season_number: 1,
    episode_number: nextId,
    duration_seconds: 1500,
    container: "mkv",
    video_codec: "h264",
    audio_codec: "aac",
    position_seconds: 0,
    completed: false,
  };
}

const slugOf = (shows: ReturnType<typeof groupShows>, libraryId: string, title: string) =>
  shows.find((show) => show.libraryId === libraryId && show.title === title)?.slug;

describe("slugify", () => {
  test("lowercases and joins words with dashes", () => {
    expect(slugify("Arcane")).toBe("arcane");
    expect(slugify("  The Office (US)  ")).toBe("the-office-us");
    expect(slugify("Star Trek: Deep Space Nine")).toBe("star-trek-deep-space-nine");
  });

  test("folds Latin accents and drops apostrophes", () => {
    expect(slugify("Pokémon Horizons")).toBe("pokemon-horizons");
    expect(slugify("Grey’s Anatomy")).toBe("greys-anatomy");
  });

  test("keeps non-Latin scripts and their marks intact", () => {
    expect(slugify("進撃の巨人")).toBe("進撃の巨人");
    expect(slugify("がっこうぐらし!")).toBe("がっこうぐらし");
    expect(slugify("Мастер и Маргарита")).toBe("мастер-и-маргарита");
    expect(slugify("पंचायत")).toBe("पंचायत");
  });

  test("falls back when nothing is left", () => {
    expect(slugify("!!!")).toBe("series");
    expect(slugify("")).toBe("series");
  });

  test("bounds length without leaving a trailing dash", () => {
    const slug = slugify(`${"a".repeat(79)} b`);
    expect(slug).toBe("a".repeat(79));
  });
});

describe("assignShowSlugs", () => {
  test("unique titles keep their bare slug", () => {
    const shows = groupShows([episode(LIB_A, "Arcane"), episode(LIB_A, "Severance")]);
    expect(slugOf(shows, LIB_A, "Arcane")).toBe("arcane");
    expect(slugOf(shows, LIB_A, "Severance")).toBe("severance");
  });

  test("same title in two libraries is disambiguated by key order", () => {
    const shows = groupShows([episode(LIB_B, "Arcane"), episode(LIB_A, "Arcane")]);
    expect(slugOf(shows, LIB_A, "Arcane")).toBe("arcane");
    expect(slugOf(shows, LIB_B, "Arcane")).toBe("arcane-2");
  });

  test("punctuation collisions never reuse another title's slug", () => {
    const slugs = assignShowSlugs([
      { key: "a\u0000Arcane", title: "Arcane" },
      { key: "a\u0000Arcane!", title: "Arcane!" },
      { key: "a\u0000Arcane 2", title: "Arcane 2" },
    ]);
    expect(slugs.get("a\u0000Arcane")).toBe("arcane");
    expect(slugs.get("a\u0000Arcane 2")).toBe("arcane-2");
    expect(slugs.get("a\u0000Arcane!")).toBe("arcane-3");
    expect(new Set(slugs.values()).size).toBe(3);
  });

  test("is independent of input order", () => {
    const shows = [
      { key: "b\u0000Arcane", title: "Arcane" },
      { key: "a\u0000Arcane", title: "Arcane" },
      { key: "a\u0000Arcane 2", title: "Arcane 2" },
    ];
    expect(assignShowSlugs(shows)).toEqual(assignShowSlugs(shows.toReversed()));
  });
});

describe("findShow", () => {
  const shows = groupShows([
    episode(LIB_A, "Arcane"),
    episode(LIB_B, "Arcane"),
    episode(LIB_A, "進撃の巨人"),
  ]);

  test("resolves slugs, including non-Latin ones", () => {
    expect(findShow(shows, "arcane")?.libraryId).toBe(LIB_A);
    expect(findShow(shows, "arcane-2")?.libraryId).toBe(LIB_B);
    expect(findShow(shows, "進撃の巨人")?.title).toBe("進撃の巨人");
  });

  test("resolves legacy internal keys", () => {
    expect(findShow(shows, `${LIB_B}\u0000Arcane`)?.slug).toBe("arcane-2");
  });

  test("tolerates a hand-typed title", () => {
    expect(findShow(shows, "Arcane")?.slug).toBe("arcane");
  });

  test("returns nothing for unknown references", () => {
    expect(findShow(shows, "severance")).toBeUndefined();
  });
});
