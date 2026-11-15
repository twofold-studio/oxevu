export const qualityPresets = [
  { id: "1080", label: "1080p · 8 Mbps", max_height: 1080, video_bitrate_mbps: 8 },
  { id: "720", label: "720p · 4 Mbps", max_height: 720, video_bitrate_mbps: 4 },
  { id: "480", label: "480p · 1.5 Mbps", max_height: 480, video_bitrate_mbps: 1.5 },
  { id: "360", label: "360p · 0.8 Mbps", max_height: 360, video_bitrate_mbps: 0.8 },
  { id: "240", label: "240p · 0.4 Mbps", max_height: 240, video_bitrate_mbps: 0.4 },
] as const;
export type QualityId = "auto" | (typeof qualityPresets)[number]["id"];
export type QualityLimit = { max_height: number; video_bitrate_mbps: number };

/** Leave room for audio, competing traffic and variation between segments. */
export function networkQuality(mbps: number): QualityLimit | undefined {
  if (!Number.isFinite(mbps) || mbps <= 0) return undefined;
  return (
    qualityPresets.find((q) => q.video_bitrate_mbps + 0.2 <= mbps * 0.7 + 0.000001) ??
    qualityPresets.at(-1)
  );
}

export function initialNetworkQuality(
  connection: { downlink?: number; saveData?: boolean } | undefined = typeof navigator ===
  "undefined"
    ? undefined
    : (
        navigator as Navigator & {
          connection?: { downlink?: number; saveData?: boolean };
        }
      ).connection,
): QualityLimit | undefined {
  if (connection?.saveData) return networkQuality(1.5);
  // Browser downlink hints are coarse/capped, even on a fast local connection.
  // Start at server limits; reduce only from actual segment transfer samples.
  return undefined;
}

export function transferMbps(bytes: number, firstByteMs: number, endMs: number): number | null {
  const elapsed = endMs - firstByteMs;
  // Tiny cached/local transfers are too short for a meaningful speed estimate.
  if (bytes <= 0 || elapsed < 20 || !Number.isFinite(elapsed)) return null;
  return (bytes * 8) / elapsed / 1000;
}

/** Automatic only steps down: three weak samples and a 30-second cooldown. */
export class AutomaticQuality {
  private weak = 0;
  private lastChange = 0;
  sample(mbps: number, current: number, now: number): QualityLimit | undefined {
    const next = networkQuality(mbps);
    if (!next || next.video_bitrate_mbps >= current || now - this.lastChange < 30_000) {
      this.weak = 0;
      return;
    }
    if (++this.weak < 3) return;
    this.weak = 0;
    this.lastChange = now;
    return next;
  }
}
