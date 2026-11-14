<script lang="ts">
  import type { Item } from "$lib/api/types";
  import { formatClock, formatCodec, formatContainer } from "$lib/media/format";

  let { item }: { item: Item } = $props();

  const rows = $derived<[string, string | null][]>([
    ["Container", formatContainer(item.container)],
    ["Video", formatCodec(item.video_codec)],
    ["Audio", formatCodec(item.audio_codec)],
    ["Duration", item.duration_seconds ? formatClock(item.duration_seconds) : null],
    ["Delivery", "Original file"],
  ]);
</script>

<section aria-labelledby="file-facts" class="flex flex-col gap-3">
  <h2 id="file-facts" class="text-sm font-medium">File</h2>
  <dl class="divide-y overflow-hidden rounded-xl border text-sm">
    {#each rows as [label, value] (label)}
      <div class="flex items-center justify-between gap-4 px-4 py-2.5">
        <dt class="text-muted-foreground">{label}</dt>
        <dd class="truncate font-mono text-xs">{value ?? "Unknown"}</dd>
      </div>
    {/each}
  </dl>
</section>
