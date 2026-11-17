<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { CatalogEpisode, MonitoredTitle } from "$lib/api/acquisition-types";
  import { errorMessage } from "$lib/api/client";
  import SeriesSelectionDialog from "./SeriesSelectionDialog.svelte";
  import { newSeriesSelection } from "./series-selection";

  let { titleId }: { titleId: number } = $props();
  let open = $state(false);
  let pending = $state(false);
  let error = $state<string | null>(null);
  let entry = $state<MonitoredTitle | null>(null);
  let episodes = $state<CatalogEpisode[]>([]);
  let generation = 0;

  async function load() {
    const current = ++generation;
    open = true;
    pending = true;
    error = null;
    entry = null;
    try {
      const [detail, { titles }] = await Promise.all([
        acquisitionApi.title("tv", titleId),
        acquisitionApi.monitored(),
      ]);
      if (current !== generation || !open) return;
      entry = titles.find(({ title }) => title.id === detail.title.id) ?? newSeriesSelection(detail.title);
      episodes = detail.episodes;
    } catch (failure) {
      if (current === generation) error = errorMessage(failure);
    } finally {
      if (current === generation) pending = false;
    }
  }
</script>

<Button variant="outline" size="lg" onclick={load}>Manage downloads</Button>
{#if open && entry}
  <SeriesSelectionDialog bind:open {entry} {episodes} />
{:else if open}
  <Dialog.Root bind:open>
    <Dialog.Content>
      <Dialog.Header><Dialog.Title>Download selection</Dialog.Title></Dialog.Header>
      {#if pending}
        <div class="grid gap-4" aria-busy="true" aria-label="Loading download selection">
          <Skeleton class="h-10 w-full" />
          <Skeleton class="h-5 w-2/3" />
        </div>
      {:else if error}
        <p role="alert" class="text-sm text-destructive">{error}</p>
        <Dialog.Footer><Button variant="outline" onclick={load}>Retry</Button></Dialog.Footer>
      {/if}
    </Dialog.Content>
  </Dialog.Root>
{/if}
