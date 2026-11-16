import type { Download } from "$lib/api/acquisition-types";

export function progress(job: Download): number {
  return job.stats.total_bytes
    ? Math.min(100, (100 * Math.max(0, job.stats.progress_bytes ?? 0)) / job.stats.total_bytes)
    : 0;
}
export function eta(job: Download): string | null {
  const remaining = (job.stats.total_bytes ?? 0) - (job.stats.progress_bytes ?? 0);
  const speed = (job.stats.live?.download_speed.mbps ?? 0) * 1024 * 1024;
  if (job.state !== "downloading" || remaining <= 0 || speed <= 0) return null;
  const seconds = Math.ceil(remaining / speed);
  if (seconds >= 3600)
    return `${Math.floor(seconds / 3600)}h ${Math.ceil((seconds % 3600) / 60)}m left`;
  return `${Math.floor(seconds / 60)}m ${seconds % 60}s left`;
}
export function transferError(raw: string): string {
  if (/429/.test(raw)) return "The indexer is rate limiting. Retry in a few minutes.";
  if (/redirect|HTTP 30[12378]/i.test(raw))
    return "This indexer returned a link Oxevu cannot fetch.";
  if (/indexer unavailable|timed? ?out|timeout/i.test(raw)) return "The indexer did not respond.";
  return raw;
}
