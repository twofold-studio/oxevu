const KEY = "oxevu.player.subtitle-delays";
const LIMIT = 200;
export const MAX_SUBTITLE_DELAY = 600;

export function normalizeSubtitleDelay(value: number): number {
  return Number.isFinite(value)
    ? Math.round(Math.max(-MAX_SUBTITLE_DELAY, Math.min(MAX_SUBTITLE_DELAY, value)) * 1000) / 1000
    : 0;
}

function entries(): [string, number][] {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    if (!Array.isArray(value)) return [];
    return value
      .filter(
        (entry): entry is [string, number] =>
          Array.isArray(entry) &&
          entry.length === 2 &&
          typeof entry[0] === "string" &&
          typeof entry[1] === "number" &&
          Number.isFinite(entry[1]) &&
          Math.abs(entry[1]) <= MAX_SUBTITLE_DELAY,
      )
      .slice(-LIMIT);
  } catch {
    return [];
  }
}

export function readSubtitleDelay(item: string, track: string): number {
  return entries().find(([key]) => key === `${item}\0${track}`)?.[1] ?? 0;
}

export function saveSubtitleDelay(item: string, track: string, seconds: number): void {
  const key = `${item}\0${track}`;
  const value = entries().filter(([id]) => id !== key);
  const delay = normalizeSubtitleDelay(seconds);
  if (delay !== 0) value.push([key, delay]);
  try {
    localStorage.setItem(KEY, JSON.stringify(value.slice(-LIMIT)));
  } catch {
    /* Browser storage may be disabled. */
  }
}
