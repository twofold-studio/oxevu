import type { Item } from "$lib/api/types";

/** Providers Oxevu has a local typographic mark for. */
export type ProviderId = "tmdb" | "imdb";

export interface RatingSource {
  /** Known provider, for its mark; null keeps the stored name as plain text. */
  id: ProviderId | null;
  /** Name to show and speak, exactly as stored or as the provider calls itself. */
  name: string;
}

export interface Rating {
  value: number;
  /** Null when the record doesn't say where the score came from. */
  source: RatingSource | null;
  /** Scope words for assistive text, such as "Season" or "Series". */
  scope?: string;
}

const PROVIDERS: Record<ProviderId, string> = { tmdb: "TMDB", imdb: "IMDb" };

/** A provider by its known id, for values whose origin is fixed in code. */
export function provider(id: ProviderId): RatingSource {
  return { id, name: PROVIDERS[id] };
}

/**
 * Resolves a stored `rating_source`. Only the stored text attributes a score;
 * provider ids or links on the record are never used to guess one.
 */
export function ratingSource(stored: string | null | undefined): RatingSource | null {
  const name = stored?.trim();
  if (!name) return null;
  const id = (Object.keys(PROVIDERS) as ProviderId[]).find((key) => key === name.toLowerCase());
  return id ? provider(id) : { id: null, name };
}

/** A 0–10 score worth showing; providers report 0 for "no votes yet". */
export function validScore(value: number | null | undefined): number | null {
  return typeof value === "number" && Number.isFinite(value) && value > 0 && value <= 10
    ? value
    : null;
}

/** Spoken and tooltip form: "TMDB season rating 8.4 out of 10". */
export function ratingLabel(rating: Rating): string {
  const subject = [rating.source?.name, rating.scope?.toLowerCase(), "rating"]
    .filter(Boolean)
    .join(" ");
  const text = `${rating.value.toFixed(1)} out of 10`;
  return `${subject.charAt(0).toUpperCase()}${subject.slice(1)} ${text}`;
}

/**
 * The rating that belongs to this record. Episodes only show a score the
 * server stored with its own source: unattributed episode scores are copies
 * of the series score, which would misrepresent the episode.
 */
export function itemRating(item: Item, series = false): Rating | null {
  if (series) {
    const value = validScore(item.show_rating);
    return value === null
      ? null
      : { value, source: ratingSource(item.show_rating_source), scope: "Series" };
  }
  const value = validScore(item.rating);
  if (value === null) return null;
  const source = ratingSource(item.rating_source);
  if (item.kind === "episode" && !source) return null;
  return { value, source };
}

/**
 * Season scores are only written by the TMDB season lookup
 * (crates/server/src/integrations/metadata.rs), so TMDB is stored evidence.
 */
export function seasonRating(item: Item | undefined): Rating | null {
  const value = validScore(item?.season_rating);
  return value === null ? null : { value, source: provider("tmdb"), scope: "Season" };
}

export interface YearSpan {
  text: string;
  label: string;
}

/**
 * Release years for the record. Episodes carry their series' premiere year
 * (there is no stored air date), so they show the series span, named as such.
 */
export function yearSpan(item: Item, series = false): YearSpan | null {
  const tv = series || item.kind === "episode";
  const start = tv ? (item.show_year ?? item.year) : item.year;
  if (!start) return null;
  const end = tv ? (item.show_end_year ?? item.end_year) : null;
  const text = end && end !== start ? `${start}–${end}` : String(start);
  const label = tv
    ? `Series aired ${end && end !== start ? `${start} to ${end}` : start}`
    : `Released ${start}`;
  return { text, label };
}

/** Placeholder values some providers send instead of omitting a field. */
export function present(value: string | null | undefined): string | null {
  const text = value?.trim();
  return text && text.toUpperCase() !== "N/A" ? text : null;
}

/** Wall-clock finish time for the rest of the item, or null without a duration. */
export function endsAt(
  duration: number | null,
  position: number | null,
  now: number,
  locale?: string,
): string | null {
  if (!duration || !Number.isFinite(duration) || duration <= 0) return null;
  const remaining = Math.max(0, duration - (position ?? 0));
  return new Date(now + remaining * 1000).toLocaleTimeString(locale, {
    hour: "numeric",
    minute: "2-digit",
  });
}
