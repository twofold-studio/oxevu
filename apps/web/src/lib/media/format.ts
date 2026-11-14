/** Clock time for the player: 4:05, 1:02:09. */
export function formatClock(totalSeconds: number): string {
  if (!Number.isFinite(totalSeconds) || totalSeconds < 0) return "0:00";
  const seconds = Math.floor(totalSeconds);
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  const ss = String(s).padStart(2, "0");
  return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${ss}` : `${m}:${ss}`;
}

/** Reading length for lists: 1h 42m, 48m, 30s. */
export function formatRuntime(totalSeconds: number | null): string | null {
  if (totalSeconds === null || !Number.isFinite(totalSeconds) || totalSeconds <= 0) return null;
  const minutes = Math.round(totalSeconds / 60);
  if (minutes < 1) return `${Math.round(totalSeconds)}s`;
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  if (h === 0) return `${m}m`;
  return m === 0 ? `${h}h` : `${h}h ${m}m`;
}

import { containerInfo } from "./container";

export function formatContainer(value: string | null): string | null {
  return containerInfo(value)?.label ?? null;
}

export function formatCodec(value: string | null): string | null {
  return value ? value.toUpperCase() : null;
}

export function plural(count: number, one: string, many = `${one}s`): string {
  return `${count} ${count === 1 ? one : many}`;
}
