import type { Item } from "$lib/api/types";
import { assignShowSlugs } from "./show-slug";

// The API lists episodes flat. Shows are grouped per library by show title so
// two libraries with the same series name stay distinct.

export interface Season {
  /** null when the scanner could not read a season number. */
  number: number | null;
  episodes: Item[];
  /**
   * Cover for the season card: the first episode art in the season (the
   * scanner resolves the nearest folder poster, so a `Season 01/poster.jpg`
   * lands here), else the show poster, else null for an honest placeholder.
   */
  posterUrl: string | null;
}

export interface Show {
  /** Internal identity: library id + title. Never shown in URLs. */
  key: string;
  /** Unique, readable URL segment; see show-slug.ts. */
  slug: string;
  title: string;
  libraryId: string;
  seasons: Season[];
  episodeCount: number;
  /** Episodes started but not finished, most useful first. */
  inProgress: Item[];
  /** First unwatched episode in order, or the first episode if all are done. */
  next: Item;
  /** Show-level art inherited by its episodes; the first episode that has it wins. */
  posterUrl: string | null;
  backdropUrl: string | null;
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

  const shows: Omit<Show, "slug">[] = [];
  for (const [key, episodes] of byShow) {
    episodes.sort(compareEpisodes);
    const seasons: Season[] = [];
    for (const episode of episodes) {
      const last = seasons.at(-1);
      if (last && last.number === episode.season_number) last.episodes.push(episode);
      else seasons.push({ number: episode.season_number, episodes: [episode], posterUrl: null });
    }
    const showPoster = episodes.find((e) => e.poster_url)?.poster_url ?? null;
    for (const season of seasons) {
      season.posterUrl = season.episodes.find((e) => e.poster_url)?.poster_url ?? showPoster;
    }
    shows.push({
      key,
      title: episodes[0].show_title ?? UNTITLED,
      libraryId: episodes[0].library_id,
      seasons,
      episodeCount: episodes.length,
      inProgress: episodes.filter((e) => e.position_seconds > 0 && !e.completed),
      next: episodes.find((e) => !e.completed) ?? episodes[0],
      posterUrl: showPoster,
      backdropUrl: episodes.find((e) => e.backdrop_url)?.backdrop_url ?? null,
    });
  }
  const slugs = assignShowSlugs(shows);
  return shows
    .map((show) => ({ ...show, slug: slugs.get(show.key) ?? show.key }))
    .sort((a, b) => collator.compare(a.title, b.title));
}

/** Stable string identity for a season within its show (tab values, URLs). */
export const seasonKey = (season: Season) => seasonKeyOf(season.number);

/** The same key from a bare season number, e.g. an episode's `season_number`. */
export const seasonKeyOf = (number: number | null) => String(number ?? "none");

/**
 * The season tab a show page should open on: the requested key when that
 * season exists (a stale or hand-edited URL falls through), else the season
 * holding `target` (what Play would start), else the first season.
 */
export function initialSeason(
  show: Show,
  requested: string | undefined,
  target: Item | undefined,
): string {
  if (requested !== undefined && show.seasons.some((s) => seasonKey(s) === requested))
    return requested;
  const holding = target && show.seasons.find((s) => s.episodes.some((e) => e.id === target.id));
  return seasonKey(holding ?? show.seasons[0]);
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

/** Compact "S1:E1", or the parts that are known. */
export function shortEpisodeCode(item: Item): string | null {
  const season = item.season_number === null ? "" : `S${item.season_number}`;
  const episode = item.episode_number === null ? "" : `E${item.episode_number}`;
  return season && episode ? `${season}:${episode}` : season || episode || null;
}

// Older scans may still carry the code in the title ("S01E01 - Pilot");
// dropping it keeps the caption from saying it twice.
const LEADING_CODE = /^\s*s\d+\s*[:.]?\s*e\d+\s*[-–—:.]?\s*/i;

/** "S1:E1 - Pilot"; just the title when the numbers are unknown. */
export function episodeCaption(item: Item): string {
  const code = shortEpisodeCode(item);
  if (!code) return item.title;
  const title = item.title.replace(LEADING_CODE, "").trim();
  return title ? `${code} - ${title}` : code;
}
