<script lang="ts">
  import LibraryIcon from "@lucide/svelte/icons/library-big";
  import ArrowRight from "@lucide/svelte/icons/arrow-right";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import Page from "$lib/components/Page.svelte";
  import CatalogGate from "$lib/components/CatalogGate.svelte";
  import { catalog } from "$lib/state/catalog.svelte";
  import { href } from "$lib/state/router.svelte";
  import PlateGrid from "$lib/features/library/PlateGrid.svelte";
  import PlateSkeleton from "$lib/features/library/PlateSkeleton.svelte";
  import MoviePlate from "$lib/features/library/MoviePlate.svelte";
  import ShowPlate from "$lib/features/library/ShowPlate.svelte";
  import EpisodeList from "$lib/features/library/EpisodeList.svelte";
  import LibraryEmpty from "$lib/features/library/LibraryEmpty.svelte";

  // Divides evenly into the 2/3/4/6-column poster grid (see PlateGrid).
  const PREVIEW = 12;
</script>

{#snippet libraryLink(url: string, title: string, count: number)}
  <Button href={url} variant="outline" class="h-auto min-w-0 justify-between gap-4 px-4 py-3">
    <span class="flex min-w-0 flex-col items-start gap-1">
      <span>{title}</span>
      <span class="text-xs font-normal text-muted-foreground">{count.toLocaleString()} in your library</span>
    </span>
    <ArrowRight aria-hidden="true" />
  </Button>
{/snippet}

<Page>
  <h1 class="mb-8 text-xl font-semibold tracking-tight">Home</h1>

  <CatalogGate>
    {#snippet skeleton()}
      <div class="flex flex-col gap-10">
        <Skeleton class="h-40 w-full rounded-xl" />
        <PlateSkeleton count={6} />
      </div>
    {/snippet}

    {#if catalog.items.length === 0}
      <LibraryEmpty
        icon={LibraryIcon}
        title="Your library is empty"
        description="Point Oxevu at a folder of movies or TV episodes, then scan it. Files are only read, never changed."
      />
    {:else}
      <div class="flex flex-col gap-10">
        <nav aria-labelledby="home-library" class="flex flex-col gap-3">
          <h2 id="home-library" class="text-base font-medium">Your library</h2>
          <div class="grid grid-cols-2 gap-3">
            {@render libraryLink(href.movies(), "All movies", catalog.movies.length)}
            {@render libraryLink(href.shows(), "All series", catalog.shows.length)}
          </div>
        </nav>
        {#if catalog.inProgress.length > 0}
          <section class="flex flex-col gap-3">
            <h2 class="text-base font-medium">Continue watching</h2>
            <EpisodeList episodes={catalog.inProgress} label="Continue watching" showContext />
          </section>
        {/if}

        {#if catalog.movies.length > 0}
          <section class="flex flex-col gap-3">
            <h2 class="text-base font-medium">Movies preview <span class="text-sm font-normal text-muted-foreground">· {Math.min(PREVIEW, catalog.movies.length)} of {catalog.movies.length.toLocaleString()}</span></h2>
            <PlateGrid label="Movies preview">
              {#each catalog.movies.slice(0, PREVIEW) as item (item.id)}<MoviePlate {item} />{/each}
            </PlateGrid>
          </section>
        {/if}

        {#if catalog.shows.length > 0}
          <section class="flex flex-col gap-3">
            <h2 class="text-base font-medium">Series preview <span class="text-sm font-normal text-muted-foreground">· {Math.min(PREVIEW, catalog.shows.length)} of {catalog.shows.length.toLocaleString()}</span></h2>
            <PlateGrid label="Series preview">
              {#each catalog.shows.slice(0, PREVIEW) as show (show.key)}<ShowPlate {show} />{/each}
            </PlateGrid>
          </section>
        {/if}
      </div>
    {/if}
  </CatalogGate>
</Page>
