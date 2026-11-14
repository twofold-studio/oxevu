<script lang="ts">
  import type { Item } from "$lib/api/types";
  import EpisodeRow from "./EpisodeRow.svelte";

  let { episodes, label, showContext = false }: { episodes: Item[]; label: string; showContext?: boolean } =
    $props();
</script>

<ul aria-label={label} class="divide-y overflow-hidden rounded-xl border">
  {#each episodes as item (item.id)}
    <EpisodeRow {item} {showContext} />
  {/each}
</ul>
