import type { RequestScope, WantedUnit } from "$lib/api/acquisition-types";
export const emptyScope = (): RequestScope => ({
  mode: "all",
  seasons: [],
  episodes: [],
  cutoff: null,
});
export function scopeLabel(scope: RequestScope) {
  if (scope.mode === "seasons") return `Seasons ${scope.seasons.join(", ")}`;
  if (scope.mode === "episodes")
    return `${scope.episodes.length} selected episode${scope.episodes.length === 1 ? "" : "s"}`;
  return {
    all: "Whole title",
    existing: "Existing episodes",
    future: "Future episodes",
    requests: "Approved requests",
  }[scope.mode];
}
export function bytes(value?: number | null) {
  if (value == null || !Number.isFinite(value)) return "Unknown size";
  if (value < 1024) return `${value} B`;
  const unit = Math.min(4, Math.floor(Math.log(value) / Math.log(1024)));
  return `${(value / 1024 ** unit).toFixed(unit > 1 ? 1 : 0)} ${["B", "KiB", "MiB", "GiB", "TiB"][unit]}`;
}
export function unitLabel(unit: WantedUnit) {
  return unit.season === null
    ? unit.name
    : `Season ${unit.season}, Episode ${unit.episode} · ${unit.name}`;
}
export function missingLabel(unit: WantedUnit, today = new Date().toISOString().slice(0, 10)) {
  if (unit.available) return "Available";
  if (unit.offline) return "Offline";
  if (!unit.wanted) return "Not monitored";
  if (!unit.air_date) return "Date unknown";
  return unit.air_date > today ? "Unaired" : "Missing";
}
