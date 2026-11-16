<script lang="ts">
  import { onMount } from "svelte";
  import RadioTowerIcon from "@lucide/svelte/icons/radio-tower";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import SettingsSection from "$lib/features/settings/SettingsSection.svelte";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { Indexer } from "$lib/api/acquisition-types";
  import { errorMessage } from "$lib/api/client";
  import IndexerDialog from "./IndexerDialog.svelte";
  const resource = createResource(async () => (await acquisitionApi.indexers()).indexers);
  let open = $state(false);let opening = $state(0);let selected = $state<Indexer | null>(null);let pending = $state<string | null>(null);let message = $state("");let error = $state<string | null>(null);
  onMount(() => { void resource.load(); });
  async function test(indexer: Indexer) { pending = indexer.id;error = null;message = "";try { await acquisitionApi.testIndexer(indexer.id);message = `${indexer.name}: connection and capabilities verified.`;await resource.load(); } catch (e) { error = errorMessage(e); } finally { pending = null; } }
</script>
<SettingsSection icon={RadioTowerIcon} title="Indexers" description="Search compatible torrent sources through Torznab. Test capabilities before relying on scheduled acquisition.">
  <ResourceGate status={resource.status} error={resource.error} what="indexers" onretry={resource.load}>
    <div class="flex flex-col gap-3">
      {#if !resource.value?.length}<p class="text-sm text-muted-foreground">No sources configured.</p>{/if}
      {#each resource.value ?? [] as indexer (indexer.id)}<div class="flex flex-wrap items-center justify-between gap-3 border-b pb-3"><div class="min-w-0"><p class="text-sm font-medium">{indexer.name}</p><Badge variant="outline">{indexer.enabled ? "Enabled" : "Disabled"}</Badge></div><div class="flex gap-2"><Button size="sm" variant="outline" disabled={pending !== null} onclick={() => test(indexer)}>{pending === indexer.id ? "Testing…" : "Test"}</Button><Button size="sm" variant="ghost" onclick={() => { selected = indexer;opening++;open = true; }}>Edit</Button></div></div>{/each}
      {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{:else if message}<p role="status" class="text-sm text-muted-foreground">{message}</p>{/if}
      <Button variant="outline" onclick={() => { selected = null;opening++;open = true; }}><PlusIcon data-icon="inline-start" />Add indexer</Button>
    </div>
  </ResourceGate>
</SettingsSection>
{#key opening}{#if open}<IndexerDialog bind:open indexer={selected} onsaved={() => { void resource.load(); }} />{/if}{/key}
