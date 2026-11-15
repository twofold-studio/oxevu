export interface ReadyRange {
  start: number;
  end: number;
}

/** Union browser and server ranges so overlapping translucent fills never stack. */
export function mergeReadyRanges(ranges: ReadyRange[], duration: number): ReadyRange[] {
  if (!Number.isFinite(duration) || duration <= 0) return [];
  const sorted = ranges
    .filter((range) => Number.isFinite(range.start) && Number.isFinite(range.end))
    .map((range) => ({ start: Math.max(0, range.start), end: Math.min(duration, range.end) }))
    .filter((range) => range.end > range.start)
    .sort((a, b) => a.start - b.start);
  const merged: ReadyRange[] = [];
  for (const range of sorted) {
    const previous = merged.at(-1);
    if (previous && range.start <= previous.end) previous.end = Math.max(previous.end, range.end);
    else merged.push(range);
  }
  return merged;
}
