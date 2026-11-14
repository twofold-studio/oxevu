import { describe, expect, test } from "vite-plus/test";
import type { Item } from "$lib/api/types";
import {
  episodeCaption,
  episodeName,
  episodeNumberLabel,
  groupShows,
  initialSeason,
} from "./grouping";

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

describe("episodeNumberLabel", () => {
  test("names season and episode in words", () => {
    expect(episodeNumberLabel(episode(2, 5))).toBe("Season 2, Episode 5");
    expect(episodeNumberLabel(episode(2, 5), { withSeason: false })).toBe("Episode 5");
  });

  test("keeps whichever numbers are known", () => {
    expect(episodeNumberLabel(episode(2, null))).toBe("Season 2");
    expect(episodeNumberLabel(episode(2, null), { withSeason: false })).toBeNull();
    expect(episodeNumberLabel(episode(null, 4))).toBe("Episode 4");
    expect(episodeNumberLabel(episode(null, null))).toBeNull();
  });

  test("labels season 0 as specials", () => {
    expect(episodeNumberLabel(episode(0, 3))).toBe("Special 3");
    expect(episodeNumberLabel(episode(0, 3), { withSeason: false })).toBe("Special 3");
    expect(episodeNumberLabel(episode(0, null))).toBe("Special");
  });
});

describe("episodeName", () => {
  test("uses the episode title", () => {
    expect(episodeName(episode(2, 5, { title: "Blisters and Bedrock" }))).toBe(
      "Blisters and Bedrock",
    );
  });

  test("drops a legacy code and falls back to the readable number", () => {
    expect(episodeName(episode(2, 5, { title: "S02E05 - Blisters and Bedrock" }))).toBe(
      "Blisters and Bedrock",
    );
    expect(episodeName(episode(2, 5, { title: "S02E05" }))).toBe("Season 2, Episode 5");
    expect(episodeName(episode(null, null, { title: "S02E05" }))).toBe("S02E05");
  });
});

describe("episodeCaption", () => {
  test("joins the readable number and the title", () => {
    expect(episodeCaption(episode(2, 5, { title: "Blisters and Bedrock" }))).toBe(
      "Season 2, Episode 5 - Blisters and Bedrock",
    );
    expect(
      episodeCaption(episode(2, 5, { title: "Blisters and Bedrock" }), { withSeason: false }),
    ).toBe("Episode 5 - Blisters and Bedrock");
  });

  test("does not repeat a code already in the title", () => {
    expect(episodeCaption(episode(1, 3, { title: "S01E03 - The Base Violence" }))).toBe(
      "Season 1, Episode 3 - The Base Violence",
    );
    expect(episodeCaption(episode(1, 3, { title: "S01E03" }))).toBe("Season 1, Episode 3");
  });

  test("handles specials and missing numbers", () => {
    expect(episodeCaption(episode(0, 2, { title: "Behind the Scenes" }))).toBe(
      "Special 2 - Behind the Scenes",
    );
    expect(episodeCaption(episode(null, null, { title: "Pilot" }))).toBe("Pilot");
  });
});

describe("initialSeason", () => {
  const [show] = groupShows([episode(1, 1), episode(1, 2), episode(2, 1), episode(null, null)]);
  const s2e1 = show.seasons[1].episodes[0];

  test("a season named in the URL wins", () => {
    expect(initialSeason(show, "1", s2e1)).toBe("1");
    expect(initialSeason(show, "none", s2e1)).toBe("none");
  });

  test("a stale or unknown season falls back to the up-next season", () => {
    expect(initialSeason(show, "7", s2e1)).toBe("2");
    expect(initialSeason(show, "", s2e1)).toBe("2");
  });

  test("with nothing to go on, the first season opens", () => {
    expect(initialSeason(show, undefined, undefined)).toBe("1");
  });
});
