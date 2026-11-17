<script lang="ts">
  import { onMount } from "svelte";
  import SearchIcon from "@lucide/svelte/icons/search";
  import * as Empty from "$lib/components/ui/empty";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { DiscoverTitle, MediaKind } from "$lib/api/acquisition-types";
  import { errorMessage } from "$lib/api/client";
  import PlateGrid from "$lib/features/library/PlateGrid.svelte";
  import PlateSkeleton from "$lib/features/library/PlateSkeleton.svelte";
  import TitlePlate from "$lib/features/library/TitlePlate.svelte";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import type { DiscoverFilters } from "$lib/media/discover-filters";
  import { href } from "$lib/state/router.svelte";

  let { query, kind, page, filters }: {
    query: string;
    kind: MediaKind;
    page: number;
    filters: DiscoverFilters;
  } = $props();

  const controller = new AbortController();
  const result = createResource(() => acquisitionApi.discover(query, kind, page, filters, controller.signal));
  let appended = $state.raw<DiscoverTitle[]>([]);
  let loadedPage = $state<number | null>(null);
  let lastPage = $state<number | null>(null);
  let loading = $state(false);
  let loadError = $state<string | null>(null);
  const titles = $derived([...(result.value?.results ?? []), ...appended]);
  const currentPage = $derived(loadedPage ?? result.value?.page ?? page);
  const hasMore = $derived(currentPage < Math.min(20, lastPage ?? result.value?.total_pages ?? page));

  onMount(() => {
    void result.load();
    return () => controller.abort();
  });

  async function loadMore() {
    if (!hasMore || loading || controller.signal.aborted) return;
    loading = true;
    loadError = null;
    const nextPage = currentPage + 1;
    try {
      const next = await acquisitionApi.discover(query, kind, nextPage, filters, controller.signal);
      if (controller.signal.aborted) return;
      const seen = new Set(titles.map(({ id }) => id));
      const fresh = next.results.filter(({ id }) => {
        if (seen.has(id)) return false;
        seen.add(id);
        return true;
      });
      appended = [...appended, ...fresh];
      loadedPage = nextPage;
      lastPage = next.total_pages;
    } catch (error) {
      if (!controller.signal.aborted) loadError = errorMessage(error);
    } finally {
      loading = false;
    }
  }

  function watchMore(element: Element) {
    if (typeof IntersectionObserver === "undefined") return;
    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting) && !loadError) void loadMore();
    }, { rootMargin: "300px" });
    observer.observe(element);
    return () => observer.disconnect();
  }
</script>

<ResourceGate status={result.status} error={result.error} what="discovery catalog" onretry={result.load}>
  {#snippet skeleton()}<PlateSkeleton count={12} />{/snippet}
  {#if titles.length === 0 && !hasMore}
    <Empty.Root class="border">
      <Empty.Header>
        <Empty.Media variant="icon"><SearchIcon /></Empty.Media>
        <Empty.Title>No matching titles</Empty.Title>
        <Empty.Description>Try a different title or change the filters.</Empty.Description>
      </Empty.Header>
    </Empty.Root>
  {:else}
    <PlateGrid label="Discover titles">
      {#each titles as title (title.id)}
        <TitlePlate href={href.discoverTitle(title.kind, title.provider_id)} title={title.title} poster={title.poster_url} detail={title.year ?? "Year unknown"}>
          {#snippet badges()}
            {#if title.availability?.items.length}<Badge variant="secondary">In library</Badge>
            {:else if title.availability?.offline}<Badge variant="outline">Offline</Badge>
            {:else if title.availability?.requested}<Badge variant="outline">Requested</Badge>{/if}
          {/snippet}
        </TitlePlate>
      {/each}
    </PlateGrid>
  {/if}
  {#if hasMore}
    {#if loading}<div class="mt-6" role="status" aria-label="Loading more titles"><PlateSkeleton count={6} /></div>{/if}
    {#key currentPage}
      <div class="mt-8 flex flex-col items-center gap-2" {@attach watchMore}>
        {#if loadError}<p role="alert" class="text-sm text-destructive">{loadError}</p>{/if}
        <Button variant="outline" disabled={loading} onclick={loadMore}>
          {loading ? "Loading more titles…" : loadError ? "Retry loading more titles" : "Load more titles"}
        </Button>
      </div>
    {/key}
  {/if}
  <p class="mt-5 text-xs text-muted-foreground">{result.value?.attribution}</p>
</ResourceGate>
