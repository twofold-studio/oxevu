<script lang="ts">
  import FilmIcon from "@lucide/svelte/icons/film";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import CatalogGate from "$lib/components/CatalogGate.svelte";
  import { catalog } from "$lib/state/catalog.svelte";
  import { plural } from "$lib/media/format";
  import PlateGrid from "./PlateGrid.svelte";
  import PlateSkeleton from "./PlateSkeleton.svelte";
  import MoviePlate from "./MoviePlate.svelte";
  import LibraryEmpty from "./LibraryEmpty.svelte";

  type Filter = "all" | "unwatched" | "watched";
  let filter = $state<Filter>("all");

  const visible = $derived(
    filter === "all"
      ? catalog.movies
      : catalog.movies.filter((m) => (filter === "watched" ? m.completed : !m.completed)),
  );
</script>

<Page>
  <PageHeader
    title="Movies"
    description={catalog.status === "ready" ? plural(catalog.movies.length, "movie") : undefined}
  >
    {#snippet actions()}
      {#if catalog.movies.length > 0}
        <ToggleGroup.Root
          type="single"
          variant="outline"
          size="sm"
          bind:value={() => filter, (v) => (filter = (v as Filter) || "all")}
          aria-label="Filter movies"
        >
          <ToggleGroup.Item value="all">All</ToggleGroup.Item>
          <ToggleGroup.Item value="unwatched">Unwatched</ToggleGroup.Item>
          <ToggleGroup.Item value="watched">Watched</ToggleGroup.Item>
        </ToggleGroup.Root>
      {/if}
    {/snippet}
  </PageHeader>

  <CatalogGate>
    {#snippet skeleton()}<PlateSkeleton />{/snippet}
    {#if catalog.movies.length === 0}
      <LibraryEmpty
        icon={FilmIcon}
        title="No movies yet"
        description="Add a movie library and scan it. Oxevu lists every file it can play."
      />
    {:else if visible.length === 0}
      <p class="rounded-xl border px-5 py-10 text-center text-sm text-muted-foreground">
        {filter === "watched" ? "You haven't finished any movies yet." : "You've watched every movie here."}
      </p>
    {:else}
      <PlateGrid label="Movies">
        {#each visible as item (item.id)}
          <MoviePlate {item} />
        {/each}
      </PlateGrid>
    {/if}
  </CatalogGate>
</Page>
