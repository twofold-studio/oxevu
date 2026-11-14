<script lang="ts">
  import SearchXIcon from "@lucide/svelte/icons/search-x";
  import * as Empty from "$lib/components/ui/empty";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import CatalogGate from "$lib/components/CatalogGate.svelte";
  import { catalog } from "$lib/state/catalog.svelte";
  import PlateGrid from "./PlateGrid.svelte";
  import PlateSkeleton from "./PlateSkeleton.svelte";
  import MoviePlate from "./MoviePlate.svelte";
  import ShowPlate from "./ShowPlate.svelte";
  import EpisodeList from "./EpisodeList.svelte";

  let { query }: { query: string } = $props();

  // Case- and accent-insensitive substring match over local catalog data.
  const fold = (value: string) =>
    value.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLocaleLowerCase();
  const needle = $derived(fold(query.trim()));
  const matches = (value: string | null) => needle !== "" && value !== null && fold(value).includes(needle);

  const movies = $derived(catalog.movies.filter((m) => matches(m.title)));
  const shows = $derived(catalog.shows.filter((s) => matches(s.title)));
  const episodes = $derived(
    catalog.shows.flatMap((s) => s.seasons.flatMap((season) => season.episodes)).filter((e) => matches(e.title)),
  );
  const total = $derived(movies.length + shows.length + episodes.length);
</script>

<Page>
  <PageHeader title={query ? `Results for “${query}”` : "Search"} />
  <CatalogGate>
    {#snippet skeleton()}<PlateSkeleton count={4} />{/snippet}
    {#if total === 0}
      <Empty.Root class="border">
        <Empty.Header>
          <Empty.Media variant="icon"><SearchXIcon /></Empty.Media>
          <Empty.Title>{query ? "No matching titles" : "Type a title to search"}</Empty.Title>
          <Empty.Description>Search looks at movie, series and episode titles.</Empty.Description>
        </Empty.Header>
      </Empty.Root>
    {:else}
      <div class="flex flex-col gap-10">
        {#if movies.length}
          <section class="flex flex-col gap-3">
            <h2 class="text-base font-medium">Movies</h2>
            <PlateGrid label="Matching movies">
              {#each movies as item (item.id)}<MoviePlate {item} />{/each}
            </PlateGrid>
          </section>
        {/if}
        {#if shows.length}
          <section class="flex flex-col gap-3">
            <h2 class="text-base font-medium">Shows</h2>
            <PlateGrid label="Matching shows">
              {#each shows as show (show.key)}<ShowPlate {show} />{/each}
            </PlateGrid>
          </section>
        {/if}
        {#if episodes.length}
          <section class="flex flex-col gap-3">
            <h2 class="text-base font-medium">Episodes</h2>
            <EpisodeList episodes={episodes} label="Matching episodes" showContext />
          </section>
        {/if}
      </div>
    {/if}
  </CatalogGate>
</Page>
