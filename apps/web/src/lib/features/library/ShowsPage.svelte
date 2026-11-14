<script lang="ts">
  import TvIcon from "@lucide/svelte/icons/tv";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import CatalogGate from "$lib/components/CatalogGate.svelte";
  import { catalog } from "$lib/state/catalog.svelte";
  import { plural } from "$lib/media/format";
  import PlateGrid from "./PlateGrid.svelte";
  import PlateSkeleton from "./PlateSkeleton.svelte";
  import ShowPlate from "./ShowPlate.svelte";
  import LibraryEmpty from "./LibraryEmpty.svelte";
</script>

<Page>
  <PageHeader
    title="Shows"
    description={catalog.status === "ready" ? plural(catalog.shows.length, "series", "series") : undefined}
  />
  <CatalogGate>
    {#snippet skeleton()}<PlateSkeleton />{/snippet}
    {#if catalog.shows.length === 0}
      <LibraryEmpty
        icon={TvIcon}
        title="No shows yet"
        description="Add a TV library and scan it. Episodes are grouped into series and seasons."
      />
    {:else}
      <PlateGrid label="Shows">
        {#each catalog.shows as show (show.key)}
          <ShowPlate {show} />
        {/each}
      </PlateGrid>
    {/if}
  </CatalogGate>
</Page>
