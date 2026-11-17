import type { Item } from "$lib/api/types";

/** Ignore the first few seconds so an accidental open doesn't count as started. */
export const MIN_RESUME_SECONDS = 10;

export const WATCHED_FRACTION = 0.9;

export function isFinished(position: number, duration: number | null): boolean {
  return duration !== null && duration > 0 && position >= duration * WATCHED_FRACTION;
}

/** Apply the same rule to previously saved progress and explicit completion. */
export function withWatchedStatus(item: Item): Item {
  return !item.completed && isFinished(item.position_seconds, item.duration_seconds)
    ? { ...item, completed: true }
    : item;
}

/** Position to resume from, or null when the item should start at zero. */
export function resumePosition(item: Item): number | null {
  if (item.completed || item.position_seconds < MIN_RESUME_SECONDS) return null;
  if (item.duration_seconds && isFinished(item.position_seconds, item.duration_seconds))
    return null;
  return item.position_seconds;
}

/** 0–1 watched fraction for progress bars; null when there is nothing to show. */
export function watchedFraction(item: Item): number | null {
  if (item.completed) return 1;
  if (!item.duration_seconds || item.position_seconds < MIN_RESUME_SECONDS) return null;
  return Math.min(item.position_seconds / item.duration_seconds, 1);
}
