<script lang="ts">
  import { onMount } from "svelte";
  import SkipBackIcon from "@lucide/svelte/icons/skip-back";
  import SkipForwardIcon from "@lucide/svelte/icons/skip-forward";
  import { Button } from "$lib/components/ui/button";
  import type { Item } from "$lib/api/types";
  import { catalog } from "$lib/state/catalog.svelte";
  import { href } from "$lib/state/router.svelte";
  import { episodeCaption } from "$lib/media/grouping";
  let { item }: { item: Item } = $props();
  onMount(() => { void catalog.ensure(); });
  const episodes = $derived(catalog.showOf(item)?.seasons.flatMap((season) => season.episodes) ?? []);
  const index = $derived(episodes.findIndex((episode) => episode.id === item.id));
  const previous = $derived(index > 0 ? episodes[index - 1] : undefined);
  const next = $derived(index >= 0 ? episodes[index + 1] : undefined);
</script>

<div class="mb-1 flex items-center justify-between gap-3" aria-label="Episode navigation">
  <Button variant="ghost" size="sm" href={previous ? href.watch(previous.id) : undefined} disabled={!previous}
    aria-label="Previous episode" title={previous ? episodeCaption(previous) : "First episode"}>
    <SkipBackIcon /> <span class="text-xs">Previous episode</span>
  </Button>
  <Button variant="ghost" size="sm" href={next ? href.watch(next.id) : undefined} disabled={!next}
    aria-label="Next episode" title={next ? episodeCaption(next) : "Last episode"}>
    <span class="text-xs">Next episode</span> <SkipForwardIcon />
  </Button>
</div>
