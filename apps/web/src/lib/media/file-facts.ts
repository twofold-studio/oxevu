import type { Item } from "$lib/api/types";
import { formatClock, formatCodec, formatContainer } from "./format";

export const factLabels = { size: "Size on disk", container: "Container", resolution: "Resolution", bitrate: "Overall bitrate", video: "Video", videoBitrate: "Video bitrate", audio: "Audio", audioBitrate: "Audio bitrate", channels: "Audio channels", duration: "Duration" } as const;
export type FactId = keyof typeof factLabels;
export interface FileFact { id: FactId; label: string; value: string | null }

function size(value: number): string {
  const unit = Math.min(4, Math.max(0, Math.floor(Math.log2(value || 1) / 10)));
  return `${(value / 1024 ** unit).toLocaleString(undefined, { maximumFractionDigits: 2 })} ${["B", "KiB", "MiB", "GiB", "TiB"][unit]}`;
}
function bitrate(value: number | null | undefined): string | null {
  if (value == null || !Number.isFinite(value) || value <= 0) return null;
  return value >= 1_000_000 ? `${(value / 1_000_000).toFixed(2)} Mbps` : `${Math.round(value / 1000)} kbps`;
}

export function fileFacts(item: Item, files?: Item[]): FileFact[] {
  const available = [...new Map((files ?? [item]).filter((i) => i.file_available !== false).map((i) => [i.id, i])).values()];
  const sizes = available.map((i) => i.size_bytes).filter((n): n is number => typeof n === "number" && Number.isFinite(n) && n >= 0);
  const distinct = (read: (i: Item) => string | null): string | null => {
    const values = [...new Set(available.map(read).filter((v): v is string => v !== null))];
    return values.length > 1 ? "Varies" : values[0] ?? null;
  };
  const values: Record<FactId, string | null> = {
    size: sizes.length ? `${sizes.length < available.length ? "At least " : ""}${size(sizes.reduce((n, v) => n + v, 0))}` : null,
    container: distinct((i) => formatContainer(i.container)),
    resolution: distinct((i) => i.video_width && i.video_height ? `${i.video_width} × ${i.video_height}` : null),
    bitrate: distinct((i) => bitrate(i.bitrate)), video: distinct((i) => formatCodec(i.video_codec)), videoBitrate: distinct((i) => bitrate(i.video_bitrate)),
    audio: distinct((i) => formatCodec(i.audio_codec)), audioBitrate: distinct((i) => bitrate(i.audio_bitrate)),
    channels: distinct((i) => i.audio_channels ? `${i.audio_channels}${i.audio_channel_layout ? ` (${i.audio_channel_layout})` : ""}` : null),
    duration: available.some((i) => i.duration_seconds != null) ? formatClock(available.reduce((n, i) => n + (i.duration_seconds ?? 0), 0)) : null,
  };
  return (Object.keys(factLabels) as FactId[]).map((id) => ({ id, label: factLabels[id], value: values[id] }));
}
