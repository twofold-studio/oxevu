<script lang="ts">
  import SearchIcon from "@lucide/svelte/icons/search";
  import { Button } from "$lib/components/ui/button";
  import type { Item } from "$lib/api/types";
  import { href } from "$lib/state/router.svelte";

  let { item }: { item: Item } = $props();

  const tmdbId = $derived.by(() => {
    const raw = item.provider_ids?.tmdb;
    if (typeof raw === "number" && raw > 0) return raw;
    if (typeof raw === "string" && /^\d+$/.test(raw) && Number(raw) > 0) return Number(raw);
    return null;
  });

  const discoverKind = $derived<"movie" | "tv">(item.kind === "episode" ? "tv" : "movie");
  const searchTitle = $derived(item.kind === "episode" ? (item.show_title ?? item.title) : item.title);

  const link = $derived(
    tmdbId !== null
      ? href.discoverTitle(discoverKind, tmdbId)
      : href.discover(searchTitle, discoverKind),
  );

  const hint = $derived(
    item.kind === "episode"
      ? "Opens the series release search. Find the season or episode you need and start a download from there."
      : null,
  );
</script>

<div class="flex flex-col gap-1">
  <Button href={link} variant="outline" size="sm">
    <SearchIcon data-icon="inline-start" />
    Find again
  </Button>
  {#if hint}
    <p class="text-xs text-muted-foreground">
      {hint}
    </p>
  {/if}
</div>
