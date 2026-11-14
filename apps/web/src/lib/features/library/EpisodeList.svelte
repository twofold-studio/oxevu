<script lang="ts">
  import type { Item } from "$lib/api/types";
  import EpisodeRow from "./EpisodeRow.svelte";

  let {
    episodes,
    label,
    showContext = false,
    upNextId,
  }: { episodes: Item[]; label: string; showContext?: boolean; upNextId?: string } = $props();
</script>

<ul aria-label={label} class="divide-y overflow-hidden rounded-xl border">
  {#each episodes as item (item.id)}
    <EpisodeRow {item} {showContext} upNext={item.id === upNextId} />
  {/each}
</ul>
