import { describe, expect, it } from "vite-plus/test";
import { isFinished, resumePosition, withWatchedStatus } from "./progress";
import type { Item } from "$lib/api/types";

const movie = {
  id: "movie", library_id: "movies", title: "Test film", kind: "movie",
  duration_seconds: 100, position_seconds: 75, completed: false,
  container: null, video_codec: null, audio_codec: null,
  show_title: null, season_number: null, episode_number: null,
} satisfies Item;

describe("watched status", () => {
  it("keeps a movie resumable at 75% and finishes it at 90%", () => {
    expect(resumePosition(movie)).toBe(75);
    expect(isFinished(89.9, 100)).toBe(false);
    expect(isFinished(90, 100)).toBe(true);
  });

  it("recognizes old saved progress and preserves explicit completion", () => {
    expect(withWatchedStatus({ ...movie, position_seconds: 90 }).completed).toBe(true);
    expect(withWatchedStatus({ ...movie, position_seconds: 0, completed: true }).completed).toBe(true);
    expect(isFinished(90, null)).toBe(false);
  });
});
