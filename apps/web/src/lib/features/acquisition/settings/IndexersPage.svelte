<script lang="ts">
  import { onMount } from "svelte";
  import RadioTowerIcon from "@lucide/svelte/icons/radio-tower";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import * as Empty from "$lib/components/ui/empty";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Spinner } from "$lib/components/ui/spinner";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import SettingsFrame from "$lib/features/settings/layout/SettingsFrame.svelte";
  import { SETTINGS_GROUP } from "$lib/features/settings/layout/group";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { Indexer } from "$lib/api/acquisition-types";
  import { errorMessage } from "$lib/api/client";
  import IndexerDialog from "./IndexerDialog.svelte";

  // Administrators and delegated indexer managers both land here.
  const resource = createResource(async () => (await acquisitionApi.indexers()).indexers);
  let open = $state(false);
  let opening = $state(0);
  let selected = $state<Indexer | null>(null);
  let testing = $state<string | null>(null);
  let message = $state("");
  let error = $state<string | null>(null);

  onMount(() => {
    void resource.load();
  });

  function edit(indexer: Indexer | null) {
    selected = indexer;
    opening++;
    open = true;
  }

  async function test(indexer: Indexer) {
    testing = indexer.id;
    error = null;
    message = "";
    try {
      await acquisitionApi.testIndexer(indexer.id);
      message = `${indexer.name}: connection and capabilities verified.`;
      await resource.load();
    } catch (e) {
      error = `${indexer.name}: ${errorMessage(e)}`;
    } finally {
      testing = null;
    }
  }

  /** Host and path only; the key is never part of the endpoint. */
  function endpointLabel(endpoint: string) {
    try {
      const url = new URL(endpoint);
      return `${url.host}${url.pathname}`;
    } catch {
      return endpoint;
    }
  }
</script>

<SettingsFrame
  page="sources/indexers"
  description="Torznab sources searched for releases, directly or through Prowlarr. Test each source before relying on scheduled acquisition."
>
  {#snippet actions()}
    {#if resource.status === "ready" && resource.value?.length}
      <Button size="sm" onclick={() => edit(null)}><PlusIcon data-icon="inline-start" />Add indexer</Button>
    {/if}
  {/snippet}

  <ResourceGate status={resource.status} error={resource.error} what="indexers" onretry={resource.load}>
    {#if !resource.value?.length}
      <Empty.Root class="border">
        <Empty.Header>
          <Empty.Media variant="icon"><RadioTowerIcon /></Empty.Media>
          <Empty.Title>No sources configured</Empty.Title>
          <Empty.Description>Add a Torznab endpoint to search for releases.</Empty.Description>
        </Empty.Header>
        <Empty.Content>
          <Button onclick={() => edit(null)}><PlusIcon data-icon="inline-start" />Add indexer</Button>
        </Empty.Content>
      </Empty.Root>
    {:else}
      <div class="flex flex-col gap-3">
        <ul class={SETTINGS_GROUP}>
          {#each resource.value as indexer (indexer.id)}
            <li class="flex flex-wrap items-center justify-between gap-3 px-5 py-4">
              <div class="flex min-w-0 flex-col gap-1">
                <div class="flex items-center gap-2">
                  <p class="truncate text-sm font-medium">{indexer.name}</p>
                  <Badge variant="outline">{indexer.enabled ? "Enabled" : "Disabled"}</Badge>
                </div>
                <p class="truncate font-mono text-xs text-muted-foreground">{endpointLabel(indexer.endpoint)}</p>
              </div>
              <div class="flex gap-2">
                <Button size="sm" variant="outline" disabled={testing !== null} onclick={() => test(indexer)}>
                  {#if testing === indexer.id}<Spinner data-icon="inline-start" />Testing…{:else}Test{/if}
                </Button>
                <Button size="sm" variant="ghost" onclick={() => edit(indexer)} aria-label={`Edit ${indexer.name}`}>Edit</Button>
              </div>
            </li>
          {/each}
        </ul>
        {#if error}
          <p role="alert" class="text-sm text-destructive">{error}</p>
        {:else if message}
          <p role="status" class="text-sm text-muted-foreground">{message}</p>
        {/if}
      </div>
    {/if}
  </ResourceGate>
</SettingsFrame>

{#key opening}
  {#if open}
    <IndexerDialog bind:open indexer={selected} onsaved={() => void resource.load()} />
  {/if}
{/key}
