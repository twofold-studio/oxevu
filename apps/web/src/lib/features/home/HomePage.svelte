<script lang="ts">
  import LibraryIcon from "@lucide/svelte/icons/library-big";
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

{#snippet sectionHeader(title: string, link?: { href: string; label: string })}
  <div class="flex items-baseline justify-between gap-4">
    <h2 class="text-base font-medium">{title}</h2>
    {#if link}
      <Button href={link.href} variant="link" size="sm" class="px-0 text-muted-foreground">{link.label}</Button>
    {/if}
  </div>
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
        {#if catalog.inProgress.length > 0}
          <section class="flex flex-col gap-3">
            {@render sectionHeader("Continue watching")}
            <EpisodeList episodes={catalog.inProgress} label="Continue watching" showContext />
          </section>
        {/if}

        {#if catalog.movies.length > 0}
          <section class="flex flex-col gap-3">
            {@render sectionHeader(
              "Movies",
              catalog.movies.length > PREVIEW ? { href: href.movies(), label: `All ${catalog.movies.length}` } : undefined,
            )}
            <PlateGrid label="Movies">
              {#each catalog.movies.slice(0, PREVIEW) as item (item.id)}<MoviePlate {item} />{/each}
            </PlateGrid>
          </section>
        {/if}

        {#if catalog.shows.length > 0}
          <section class="flex flex-col gap-3">
            {@render sectionHeader(
              "Shows",
              catalog.shows.length > PREVIEW ? { href: href.shows(), label: `All ${catalog.shows.length}` } : undefined,
            )}
            <PlateGrid label="Shows">
              {#each catalog.shows.slice(0, PREVIEW) as show (show.key)}<ShowPlate {show} />{/each}
            </PlateGrid>
          </section>
        {/if}
      </div>
    {/if}
  </CatalogGate>
</Page>
