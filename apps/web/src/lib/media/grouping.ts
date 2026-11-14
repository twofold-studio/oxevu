import type { Item } from "$lib/api/types";

// The API lists episodes flat. Shows are grouped per library by show title so
// two libraries with the same series name stay distinct.

export interface Season {
  /** null when the scanner could not read a season number. */
  number: number | null;
  episodes: Item[];
}

export interface Show {
  key: string;
  title: string;
  libraryId: string;
  seasons: Season[];
  episodeCount: number;
  /** Episodes started but not finished, most useful first. */
  inProgress: Item[];
  /** First unwatched episode in order, or the first episode if all are done. */
  next: Item;
}

const UNTITLED = "Untitled series";

export function showKey(item: Item): string {
  return `${item.library_id}\u0000${item.show_title ?? UNTITLED}`;
}

const compareNumber = (a: number | null, b: number | null) =>
  (a ?? Number.MAX_SAFE_INTEGER) - (b ?? Number.MAX_SAFE_INTEGER);

export const collator = new Intl.Collator(undefined, { sensitivity: "base", numeric: true });

function compareEpisodes(a: Item, b: Item) {
  return (
    compareNumber(a.season_number, b.season_number) ||
    compareNumber(a.episode_number, b.episode_number) ||
    collator.compare(a.title, b.title)
  );
}

export function groupShows(items: Item[]): Show[] {
  const byShow = new Map<string, Item[]>();
  for (const item of items) {
    if (item.kind !== "episode") continue;
    const key = showKey(item);
    const list = byShow.get(key);
    if (list) list.push(item);
    else byShow.set(key, [item]);
  }

  const shows: Show[] = [];
  for (const [key, episodes] of byShow) {
    episodes.sort(compareEpisodes);
    const seasons: Season[] = [];
    for (const episode of episodes) {
      const last = seasons.at(-1);
      if (last && last.number === episode.season_number) last.episodes.push(episode);
      else seasons.push({ number: episode.season_number, episodes: [episode] });
    }
    shows.push({
      key,
      title: episodes[0].show_title ?? UNTITLED,
      libraryId: episodes[0].library_id,
      seasons,
      episodeCount: episodes.length,
      inProgress: episodes.filter((e) => e.position_seconds > 0 && !e.completed),
      next: episodes.find((e) => !e.completed) ?? episodes[0],
    });
  }
  return shows.sort((a, b) => collator.compare(a.title, b.title));
}

export function seasonLabel(number: number | null): string {
  if (number === null) return "Unnumbered";
  if (number === 0) return "Specials";
  return `Season ${number}`;
}

/** "S01E04", or the parts that are known. */
export function episodeCode(item: Item): string | null {
  const pad = (value: number) => String(value).padStart(2, "0");
  const season = item.season_number === null ? "" : `S${pad(item.season_number)}`;
  const episode = item.episode_number === null ? "" : `E${pad(item.episode_number)}`;
  return season || episode ? season + episode : null;
}
