<script lang="ts">
  import { onMount } from "svelte";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import * as Empty from "$lib/components/ui/empty";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { errorMessage } from "$lib/api/client";
  import type { Download } from "$lib/api/acquisition-types";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import { href } from "$lib/state/router.svelte";
  import ActivityTransfer from "./ActivityTransfer.svelte";
  import AddDownloadDialog from "./AddDownloadDialog.svelte";
  import DownloadFilesDialog from "./DownloadFilesDialog.svelte";
  import ImportDialog from "./ImportDialog.svelte";
  import ActivityLimits from "./ActivityLimits.svelte";
  import { session } from "$lib/state/session.svelte";
  const resource = createResource(async () => { const [d,e] = await Promise.all([acquisitionApi.downloads(), acquisitionApi.events()]); return { downloads: d.downloads, events: e.events, importEnabled: d.import_enabled }; });
  let error = $state<string | null>(null);let pending = $state<string | null>(null);let addOpen = $state(false);let filesOpen = $state(false);let importOpen = $state(false);let removeOpen = $state(false);let selectedId = $state<string | null>(null);let opening = $state(0);
  const selected = $derived(resource.value?.downloads.find((d) => d.id === selectedId) ?? null);
  onMount(() => { void resource.load(); const timer = setInterval(() => { if (document.visibilityState === "visible") void resource.load(); }, 2500); return () => clearInterval(timer); });
  async function action(job: Download, value: string) { pending = job.id;error = null;try { await acquisitionApi.control(job.id, value); removeOpen = false; await resource.load(); } catch (e) { error = errorMessage(e); } finally { pending = null; } }
</script>
<Page class="max-w-6xl">
  <PageHeader title="Activity" description="Transfers, seeding and import history in one place.">
    {#snippet actions()}<Button variant="outline" onclick={resource.load} aria-label="Refresh activity"><RotateCwIcon /></Button><Button onclick={() => { opening++; addOpen = true; }}><PlusIcon data-icon="inline-start" />Add download</Button>{/snippet}
  </PageHeader>
  {#if session.user?.is_admin}<ActivityLimits />{/if}
  {#if error}<p role="alert" class="mb-4 text-sm text-destructive">{error}</p>{/if}
  <ResourceGate status={resource.status} error={resource.error} what="download activity" onretry={resource.load}>
    {#if resource.value?.downloads.length === 0}<Empty.Root class="border"><Empty.Header><Empty.Media variant="icon"><DownloadIcon /></Empty.Media><Empty.Title>No transfers</Empty.Title><Empty.Description>Find a release under Wanted, or add a torrent directly. Configure storage and indexers in download settings.</Empty.Description></Empty.Header><Empty.Content><Button variant="outline" href={href.settings("acquisition")}>Download settings</Button></Empty.Content></Empty.Root>
    {:else}
      <ul class="divide-y rounded-xl border bg-card">
        {#each resource.value?.downloads ?? [] as job (job.id)}
          <ActivityTransfer {job} importEnabled={resource.value?.importEnabled ?? false} pending={pending === job.id} onaction={(value) => void action(job, value)}
            onfiles={() => { selectedId = job.id; opening++; filesOpen = true; }}
            onimport={() => { selectedId = job.id; opening++; importOpen = true; }}
            onremove={() => { selectedId = job.id; removeOpen = true; }} />
        {/each}
      </ul>
    {/if}
    {#if resource.value?.events.length}<details class="mt-6 rounded-xl border"><summary class="cursor-pointer px-4 py-3 text-sm font-medium">Recent events</summary><ul class="divide-y border-t">{#each resource.value.events as event (event.id)}<li class="flex flex-wrap justify-between gap-2 px-4 py-3 text-sm"><span>{event.message}{#if resource.value.downloads.some((job) => job.id === event.entity_id)} <a class="font-mono text-xs underline" title={event.entity_id} href={href.activity()} onclick={(eventClick) => { eventClick.preventDefault(); document.getElementById(`download-${event.entity_id}`)?.scrollIntoView({ behavior: "smooth", block: "center" }); }}>Job {event.entity_id.slice(0,8)}</a>{/if}</span><time class="text-xs text-muted-foreground">{new Date(event.created_at * 1000).toLocaleString()}</time></li>{/each}</ul></details>{/if}
  </ResourceGate>
</Page>
{#key opening}
  {#if addOpen}<AddDownloadDialog bind:open={addOpen} onadded={() => { void resource.load(); }} />{/if}
  {#if selected && filesOpen}<DownloadFilesDialog bind:open={filesOpen} job={selected} onsaved={() => { void resource.load(); }} onimport={() => { filesOpen = false; importOpen = true; }} />
  {:else if selected && importOpen}<ImportDialog bind:open={importOpen} job={selected} onimported={() => { void resource.load(); }} />{/if}
{/key}
<AlertDialog.Root bind:open={removeOpen}><AlertDialog.Content><AlertDialog.Header><AlertDialog.Title>Remove this transfer?</AlertDialog.Title><AlertDialog.Description>The queue entry is removed. Downloaded files, seeding sources and library copies are kept.</AlertDialog.Description></AlertDialog.Header><AlertDialog.Footer><AlertDialog.Cancel>Keep transfer</AlertDialog.Cancel><AlertDialog.Action onclick={() => selected && action(selected, "remove")}>Remove job, keep files</AlertDialog.Action></AlertDialog.Footer></AlertDialog.Content></AlertDialog.Root>
