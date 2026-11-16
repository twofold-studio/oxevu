<script lang="ts">
  import type { Item } from "$lib/api/types";
  import { formatClock, formatCodec, formatContainer } from "$lib/media/format";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";

  let { item }: { item: Item } = $props();

  function size(value: number | null | undefined): string | null {
    if (value == null || !Number.isFinite(value) || value < 0) return null;
    const unit = value >= 1024 ** 3 ? 3 : value >= 1024 ** 2 ? 2 : value >= 1024 ? 1 : 0;
    return `${(value / 1024 ** unit).toLocaleString(undefined, { maximumFractionDigits: 2 })} ${["B", "KiB", "MiB", "GiB"][unit]}`;
  }

  function bitrate(value: number | null | undefined): string | null {
    if (value == null || !Number.isFinite(value) || value <= 0) return null;
    return value >= 1_000_000 ? `${(value / 1_000_000).toFixed(2)} Mbps` : `${Math.round(value / 1000)} kbps`;
  }

  const rows = $derived<[string, string | null][]>([
    ["Size on disk", size(item.size_bytes)],
    ["Container", formatContainer(item.container)],
    ["Resolution", item.video_width && item.video_height ? `${item.video_width} × ${item.video_height}` : null],
    ["Overall bitrate", bitrate(item.bitrate)],
    ["Video", formatCodec(item.video_codec)],
    ["Video bitrate", bitrate(item.video_bitrate)],
    ["Audio", formatCodec(item.audio_codec)],
    ["Audio bitrate", bitrate(item.audio_bitrate)],
    ["Audio channels", item.audio_channels ? `${item.audio_channels}${item.audio_channel_layout ? ` (${item.audio_channel_layout})` : ""}` : null],
    ["Duration", item.duration_seconds ? formatClock(item.duration_seconds) : null],
  ]);
</script>

<details class="group flex flex-col gap-3">
  <summary class="flex cursor-pointer list-none items-center justify-between gap-3 rounded-md py-2 text-sm font-medium focus-visible:outline-2 focus-visible:outline-ring [&::-webkit-details-marker]:hidden">
    Stats for nerds
    <ChevronDownIcon class="size-4 text-muted-foreground transition-transform group-open:rotate-180" aria-hidden="true" />
  </summary>
  <dl class="divide-y overflow-hidden rounded-xl border text-sm">
    {#each rows as [label, value] (label)}
      <div class="flex items-center justify-between gap-4 px-4 py-2.5">
        <dt class="text-muted-foreground">{label}</dt>
        <dd class="truncate font-mono text-xs">{value ?? "Unknown"}</dd>
      </div>
    {/each}
  </dl>
  <p class="mt-2 text-xs text-muted-foreground">Source file details. Unknown values were not reported by the media probe; rescan older library entries to populate them.</p>
</details>
