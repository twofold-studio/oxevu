<script lang="ts">
  import { onMount } from "svelte";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import PauseIcon from "@lucide/svelte/icons/pause";
  import PlayIcon from "@lucide/svelte/icons/play";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import FilesIcon from "@lucide/svelte/icons/files";
  import TrashIcon from "@lucide/svelte/icons/trash-2";
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
  import { bytes } from "./format";
  import AddDownloadDialog from "./AddDownloadDialog.svelte";
  import DownloadFilesDialog from "./DownloadFilesDialog.svelte";
  import ImportDialog from "./ImportDialog.svelte";
  const resource = createResource(async () => { const [d,e] = await Promise.all([acquisitionApi.downloads(), acquisitionApi.events()]); return { downloads: d.downloads, events: e.events }; });
  let error = $state<string | null>(null);let pending = $state<string | null>(null);let addOpen = $state(false);let filesOpen = $state(false);let importOpen = $state(false);let removeOpen = $state(false);let selectedId = $state<string | null>(null);let opening = $state(0);
  const selected = $derived(resource.value?.downloads.find((d) => d.id === selectedId) ?? null);
  onMount(() => { void resource.load(); const timer = setInterval(() => { if (document.visibilityState === "visible") void resource.load(); }, 2500); return () => clearInterval(timer); });
  async function action(job: Download, value: string) { pending = job.id;error = null;try { await acquisitionApi.control(job.id, value); removeOpen = false; await resource.load(); } catch (e) { error = errorMessage(e); } finally { pending = null; } }
  function progress(job: Download) { return job.stats.total_bytes ? Math.min(100, 100 * (job.stats.progress_bytes ?? 0) / job.stats.total_bytes) : 0; }
</script>
<Page class="max-w-6xl">
  <PageHeader title="Activity" description="Transfers, seeding and import history in one place.">
    {#snippet actions()}<Button variant="outline" onclick={resource.load} aria-label="Refresh activity"><RotateCwIcon /></Button><Button onclick={() => { opening++; addOpen = true; }}><PlusIcon data-icon="inline-start" />Add download</Button>{/snippet}
  </PageHeader>
  {#if error}<p role="alert" class="mb-4 text-sm text-destructive">{error}</p>{/if}
  <ResourceGate status={resource.status} error={resource.error} what="download activity" onretry={resource.load}>
    {#if resource.value?.downloads.length === 0}<Empty.Root class="border"><Empty.Header><Empty.Media variant="icon"><DownloadIcon /></Empty.Media><Empty.Title>No transfers</Empty.Title><Empty.Description>Find a release under Wanted, or add a torrent directly. Configure storage and indexers in download settings.</Empty.Description></Empty.Header><Empty.Content><Button variant="outline" href={href.settings("acquisition")}>Download settings</Button></Empty.Content></Empty.Root>
    {:else}
      <ul class="divide-y rounded-xl border bg-card">
        {#each resource.value?.downloads ?? [] as job (job.id)}
          <li class="flex flex-col gap-3 p-4">
            <div class="flex flex-wrap items-start justify-between gap-3"><div class="min-w-0 flex-1"><h2 class="font-medium break-words">{job.title?.title ?? job.files[0]?.path ?? "Resolving torrent metadata"}</h2><p class="mt-1 text-xs text-muted-foreground">{bytes(job.stats.progress_bytes ?? 0)} / {bytes(job.stats.total_bytes)} · Uploaded {bytes(job.stats.uploaded_bytes ?? 0)}</p></div><Badge variant={job.state === "error" ? "outline" : "secondary"} class="capitalize">{job.state}</Badge></div>
            <div role="progressbar" aria-label={`Download progress for ${job.title?.title ?? "torrent"}`} aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(progress(job))} class="h-1.5 overflow-hidden rounded-full bg-muted"><div class="h-full bg-primary transition-[width] duration-300" style:width={`${progress(job)}%`}></div></div>
            <div class="flex flex-wrap items-center justify-between gap-3"><span class="text-xs text-muted-foreground">↓ {bytes((job.stats.live?.download_speed.mbps ?? 0) * 1024 * 1024)}/s · ↑ {bytes((job.stats.live?.upload_speed.mbps ?? 0) * 1024 * 1024)}/s · {job.stats.live?.snapshot?.peer_stats?.live ?? 0} peers</span><div class="flex flex-wrap gap-1">
              {#if ["paused", "queued", "completed"].includes(job.state)}<Button variant="ghost" size="icon-sm" aria-label="Resume transfer" disabled={pending === job.id} onclick={() => action(job, "resume")}><PlayIcon /></Button>{:else if job.state === "error"}<Button variant="ghost" size="icon-sm" aria-label="Retry transfer" disabled={pending === job.id} onclick={() => action(job, "retry")}><RotateCwIcon /></Button>{:else}<Button variant="ghost" size="icon-sm" aria-label="Pause transfer" disabled={pending === job.id} onclick={() => action(job, "pause")}><PauseIcon /></Button>{/if}
              <Button variant="ghost" size="icon-sm" aria-label="Inspect files and selection" onclick={() => { selectedId = job.id; opening++; filesOpen = true; }}><FilesIcon /></Button>
              <Button variant="ghost" size="icon-sm" aria-label="Remove job and keep files" onclick={() => { selectedId = job.id; removeOpen = true; }}><TrashIcon /></Button>
            </div></div>
            {#if job.error}<p class="text-sm text-destructive">{job.error}</p>{/if}
            {#if job.import_warning}<p class="text-sm text-muted-foreground">Import review · {job.import_warning}</p>{/if}
            {#if job.queue_reason}<p class="text-xs text-muted-foreground">{job.queue_reason}</p>{/if}
          </li>
        {/each}
      </ul>
    {/if}
    {#if resource.value?.events.length}<details class="mt-6 rounded-xl border"><summary class="cursor-pointer px-4 py-3 text-sm font-medium">Recent events</summary><ul class="divide-y border-t">{#each resource.value.events as event (event.id)}<li class="flex flex-wrap justify-between gap-2 px-4 py-3 text-sm"><span>{event.message}</span><time class="text-xs text-muted-foreground">{new Date(event.created_at * 1000).toLocaleString()}</time></li>{/each}</ul></details>{/if}
  </ResourceGate>
</Page>
{#key opening}
  {#if addOpen}<AddDownloadDialog bind:open={addOpen} onadded={() => { void resource.load(); }} />{/if}
  {#if selected && filesOpen}<DownloadFilesDialog bind:open={filesOpen} job={selected} onsaved={() => { void resource.load(); }} onimport={() => { filesOpen = false; importOpen = true; }} />
  {:else if selected && importOpen}<ImportDialog bind:open={importOpen} job={selected} onimported={() => { void resource.load(); }} />{/if}
{/key}
<AlertDialog.Root bind:open={removeOpen}><AlertDialog.Content><AlertDialog.Header><AlertDialog.Title>Remove this transfer?</AlertDialog.Title><AlertDialog.Description>The queue entry is removed. Downloaded files, seeding sources and library copies are kept.</AlertDialog.Description></AlertDialog.Header><AlertDialog.Footer><AlertDialog.Cancel>Keep transfer</AlertDialog.Cancel><AlertDialog.Action onclick={() => selected && action(selected, "remove")}>Remove job, keep files</AlertDialog.Action></AlertDialog.Footer></AlertDialog.Content></AlertDialog.Root>
