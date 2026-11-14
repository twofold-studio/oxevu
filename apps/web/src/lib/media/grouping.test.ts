import { describe, expect, test } from "vite-plus/test";
import type { Item } from "$lib/api/types";
import { episodeCaption, groupShows } from "./grouping";

const LIB = "11111111-1111-4111-8111-111111111111";
const art = (path: string) => `/api/v1/items/${path}/artwork/poster`;

let nextId = 0;
function episode(season: number | null, number: number | null, extra: Partial<Item> = {}): Item {
  nextId += 1;
  return {
    id: `item-${nextId}`,
    library_id: LIB,
    title: `Episode ${nextId}`,
    kind: "episode",
    show_title: "Show",
    season_number: season,
    episode_number: number,
    duration_seconds: 1500,
    container: "mkv",
    video_codec: "h264",
    audio_codec: "aac",
    position_seconds: 0,
    completed: false,
    ...extra,
  };
}

describe("groupShows seasons", () => {
  test("counts each season once, whatever the input order", () => {
    const [show] = groupShows([
      episode(2, 1),
      episode(1, 2),
      episode(2, 2),
      episode(1, 1),
      episode(2, 3),
    ]);
    expect(show.seasons.map((s) => [s.number, s.episodes.length])).toEqual([
      [1, 2],
      [2, 3],
    ]);
    expect(show.episodeCount).toBe(5);
  });

  test("season covers prefer season art, then the show poster, then nothing", () => {
    const [withArt] = groupShows([
      episode(1, 1, { poster_url: art("s1") }),
      episode(2, 1),
      episode(3, 1, { poster_url: art("s3") }),
    ]);
    expect(withArt.seasons.map((s) => s.posterUrl)).toEqual([art("s1"), art("s1"), art("s3")]);

    const [bare] = groupShows([episode(1, 1, { show_title: "Bare" })]);
    expect(bare.seasons[0].posterUrl).toBeNull();
  });
});

describe("episodeCaption", () => {
  test("joins a compact code and the title", () => {
    expect(episodeCaption(episode(1, 1, { title: "Welcome to the Playground" }))).toBe(
      "S1:E1 - Welcome to the Playground",
    );
  });

  test("does not repeat a code already in the title", () => {
    expect(episodeCaption(episode(1, 3, { title: "S01E03 - The Base Violence" }))).toBe(
      "S1:E3 - The Base Violence",
    );
    expect(episodeCaption(episode(1, 3, { title: "S01E03" }))).toBe("S1:E3");
  });

  test("falls back to the title when numbers are unknown", () => {
    expect(episodeCaption(episode(null, null, { title: "Pilot" }))).toBe("Pilot");
  });
});
