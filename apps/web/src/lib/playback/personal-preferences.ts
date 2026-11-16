// Browser-local preferences: never part of a party snapshot or control.
const VOLUME = "oxevu.player.volume";
const SUBTITLES = "oxevu.player.subtitles";
export type SubtitlePreference = { off: true } | { language: string | null; label: string };

function read(key: string): unknown {
  try {
    return JSON.parse(localStorage.getItem(key) ?? "null");
  } catch {
    return null;
  }
}
function write(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    /* Storage may be disabled. */
  }
}

export function readVolume(): { level: number; muted: boolean } {
  const value = read(VOLUME) as { level?: unknown; muted?: unknown } | null;
  return value &&
    typeof value.level === "number" &&
    Number.isFinite(value.level) &&
    value.level >= 0 &&
    value.level <= 1 &&
    typeof value.muted === "boolean"
    ? { level: value.level, muted: value.muted }
    : { level: 1, muted: false };
}
export function saveVolume(level: number, muted: boolean) {
  write(VOLUME, { level, muted });
}
export function readSubtitlePreference(): SubtitlePreference | null {
  const value = read(SUBTITLES) as { off?: unknown; language?: unknown; label?: unknown } | null;
  if (value?.off === true) return { off: true };
  if (
    value &&
    (value.language === null || typeof value.language === "string") &&
    typeof value.label === "string"
  )
    return { language: value.language, label: value.label };
  return null;
}
export function saveSubtitlePreference(value: SubtitlePreference) {
  write(SUBTITLES, value);
}
