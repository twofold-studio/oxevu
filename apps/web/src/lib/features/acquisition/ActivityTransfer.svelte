<script lang="ts">
  import ArrowDownIcon from "@lucide/svelte/icons/arrow-down";
  import ArrowUpIcon from "@lucide/svelte/icons/arrow-up";
  import CircleCheckIcon from "@lucide/svelte/icons/circle-check";
  import CircleXIcon from "@lucide/svelte/icons/circle-x";
  import ClockIcon from "@lucide/svelte/icons/clock";
  import SearchIcon from "@lucide/svelte/icons/search";
  import PauseIcon from "@lucide/svelte/icons/pause";
  import PlayIcon from "@lucide/svelte/icons/play";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import FilesIcon from "@lucide/svelte/icons/files";
  import TrashIcon from "@lucide/svelte/icons/trash-2";
  import GaugeIcon from "@lucide/svelte/icons/gauge";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import { session } from "$lib/state/session.svelte";
  import { href } from "$lib/state/router.svelte";
  import type { Download } from "$lib/api/acquisition-types";
  import { bytes } from "./format";
  import { progress, eta, transferError, importMessage, recognitionLabel } from "./activity";
  import TransferLimit from "./TransferLimit.svelte";
  import DownloadIdentifyDialog from "./DownloadIdentifyDialog.svelte";

  let { job, importEnabled, pending, onaction, onfiles, onimport, onremove, onrefresh }: {
    job: Download; importEnabled: boolean; pending: boolean;
    onaction: (action: string) => void; onfiles: () => void; onimport: () => void; onremove: () => void; onrefresh: () => void;
  } = $props();
  let identifyOpen = $state(false);
  let limitOpen = $state(false);
  let identifyOpening = $state(0);
  const label = $derived(job.release_title ?? job.files[0]?.path ?? job.title?.title ?? "Resolving torrent metadata");
  const downloaded = $derived(["seeding", "completed"].includes(job.state) || job.stats.finished === true);
  const left = $derived(eta(job));
  const stateLabel = $derived(
    ({ downloading: "Downloading", seeding: "Seeding", completed: "Completed", paused: "Paused", queued: "Queued", error: "Error" } as Record<string, string>)[job.state] ?? job.state,
  );
</script>

<li class="flex flex-col gap-2 p-4" id={`download-${job.id}`}>
  <div class="flex items-start gap-2">
    <Tooltip.Root>
      <Tooltip.Trigger class="mt-0.5 inline-flex shrink-0" aria-label={stateLabel}>
          {#if job.state === "downloading"}<ArrowDownIcon class="size-4 text-primary" />
          {:else if job.state === "seeding"}<ArrowUpIcon class="size-4 text-green-500" />
          {:else if job.state === "completed"}<CircleCheckIcon class="size-4 text-green-500" />
          {:else if job.state === "paused"}<PauseIcon class="size-4 text-muted-foreground" />
          {:else if job.state === "queued"}<ClockIcon class="size-4 text-muted-foreground" />
          {:else if job.state === "error"}<CircleXIcon class="size-4 text-destructive" />
          {:else}<ClockIcon class="size-4 text-muted-foreground" />{/if}
      </Tooltip.Trigger>
      <Tooltip.Content>{stateLabel}</Tooltip.Content>
    </Tooltip.Root>
    <div class="min-w-0 flex-1">
      <h2 class="text-sm font-medium break-words">{label}</h2>
      <p class="mt-0.5 text-xs text-muted-foreground">{[job.title?.title !== label ? job.title?.title : null, job.indexer_name, job.release_size ? bytes(job.release_size) : null].filter(Boolean).join(" · ")}{#if job.id} · <span class="font-mono">{job.id.slice(0,8)}</span>{/if}</p>
    </div>
    <div class="flex shrink-0 items-center gap-0.5">
      {#if ["paused","queued","completed"].includes(job.state)}<Button variant="ghost" size="icon-sm" aria-label="Resume" disabled={pending} onclick={() => onaction("resume")}><PlayIcon /></Button>
      {:else if job.state === "error"}<Button variant="ghost" size="icon-sm" aria-label="Retry" disabled={pending} onclick={() => onaction("retry")}><RotateCwIcon /></Button>
      {:else}<Button variant="ghost" size="icon-sm" aria-label="Pause" disabled={pending} onclick={() => onaction("pause")}><PauseIcon /></Button>{/if}
      <Button variant="ghost" size="icon-sm" aria-label="Files" onclick={onfiles}><FilesIcon /></Button>
      <Button variant="ghost" size="icon-sm" aria-label="Transfer speed limit" title="Transfer speed limit" aria-expanded={limitOpen} onclick={() => (limitOpen = !limitOpen)}><GaugeIcon /></Button>
      {#if downloaded && session.user?.is_admin}<Button size="sm" variant="outline" onclick={onimport}>Import</Button>{/if}
      <Button variant="ghost" size="icon-sm" aria-label="Remove" onclick={onremove}><TrashIcon /></Button>
    </div>
  </div>

  <div class="flex items-center gap-2 text-xs text-muted-foreground">
    <span class="w-8 shrink-0 text-right tabular-nums">{Math.round(progress(job))}%</span>
    <div role="progressbar" aria-label={`Download progress for ${label}`} aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(progress(job))} class="h-1.5 flex-1 overflow-hidden rounded-full bg-muted"><div class="h-full bg-primary transition-[width] duration-300" style:width={`${progress(job)}%`}></div></div>
    <span class="shrink-0 tabular-nums">{bytes(job.stats.progress_bytes ?? 0)} / {bytes(job.stats.total_bytes)}</span>
  </div>

  <div class="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 text-xs text-muted-foreground">
    <span>↓ {bytes((job.stats.live?.download_speed.mbps ?? 0)*1024*1024)}/s · ↑ {bytes((job.stats.live?.upload_speed.mbps ?? 0)*1024*1024)}/s · {job.stats.live?.snapshot?.peer_stats?.live ?? 0} peers{left ? ` · ${left}` : ""}{#if job.stats.uploaded_bytes} · ↑ {bytes(job.stats.uploaded_bytes)}{/if}</span>
    {#if job.recognition}
      {@const recognition = job.recognition}
      <span class="inline-flex items-center gap-1">
        <Tooltip.Root>
          <Tooltip.Trigger aria-label={recognitionLabel(recognition.state)} class="inline-flex">
              <Badge variant={recognition.state === "needs_review" ? "outline" : recognition.state === "matched" ? "secondary" : "ghost"}>
                {#if recognition.state === "matched"}<CircleCheckIcon data-icon="inline-start" class="size-3" />
                {:else if recognition.state === "needs_review"}<SearchIcon data-icon="inline-start" class="size-3" />
                {:else if recognition.state === "pending"}<ClockIcon data-icon="inline-start" class="size-3" />
                {:else}<CircleXIcon data-icon="inline-start" class="size-3" />{/if}
                {recognitionLabel(recognition.state)}
              </Badge>
          </Tooltip.Trigger>
          <Tooltip.Content>{recognition.message ?? recognitionLabel(recognition.state)}</Tooltip.Content>
        </Tooltip.Root>
        {#if (recognition.state === "needs_review" || recognition.state === "unavailable") && session.user?.is_admin}
          <Button variant="ghost" size="sm" class="h-5 px-1.5 text-xs" onclick={() => { identifyOpening++; identifyOpen = true; }}>
            <SearchIcon class="size-3" />Identify
          </Button>
        {/if}
      </span>
    {/if}
  </div>

  {#if limitOpen}<TransferLimit id={job.id} initial={job.download_limit_kib ?? 0} />{/if}
  {#if job.error}
    <p role="alert" class="text-sm text-destructive">{transferError(job.error)}</p>
    <details class="text-xs text-muted-foreground"><summary class="cursor-pointer">Technical details</summary><p class="mt-2 break-words">{job.error}</p></details>
  {/if}
  {#if downloaded}
    <p class="flex flex-wrap items-center gap-1.5 text-xs text-muted-foreground" title={importMessage(job, importEnabled)}>
      {#if job.imported}<CircleCheckIcon class="size-3.5" />Imported
      {:else if !importEnabled}Imports off
      {:else}{job.import_blocker ?? job.import_warning ?? "Import pending"}{/if}
      {#if !job.imported && session.user?.is_admin}
        <a class="underline" href={href.settings("downloads/imports")}>Import settings</a>
      {/if}
    </p>
  {:else if job.import_warning}<p class="text-sm text-muted-foreground">Import review · {job.import_warning}</p>{/if}
  {#if job.queue_reason}<p class="text-xs text-muted-foreground">{job.queue_reason}</p>{/if}
</li>
{#key identifyOpening}
  {#if identifyOpen}<DownloadIdentifyDialog bind:open={identifyOpen} {job} onidentified={onrefresh} />{/if}
{/key}
