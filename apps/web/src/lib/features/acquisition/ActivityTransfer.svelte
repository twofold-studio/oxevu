<script lang="ts">
  import PauseIcon from "@lucide/svelte/icons/pause";
  import PlayIcon from "@lucide/svelte/icons/play";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import FilesIcon from "@lucide/svelte/icons/files";
  import TrashIcon from "@lucide/svelte/icons/trash-2";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { session } from "$lib/state/session.svelte";
  import { href } from "$lib/state/router.svelte";
  import type { Download } from "$lib/api/acquisition-types";
  import { bytes } from "./format";
  import { progress, eta, transferError, importMessage } from "./activity";
  import TransferLimit from "./TransferLimit.svelte";

  let { job, importEnabled, pending, onaction, onfiles, onimport, onremove }: {
    job: Download; importEnabled: boolean; pending: boolean;
    onaction: (action: string) => void; onfiles: () => void; onimport: () => void; onremove: () => void;
  } = $props();
  const label = $derived(job.release_title ?? job.files[0]?.path ?? job.title?.title ?? "Resolving torrent metadata");
  const downloaded = $derived(["seeding", "completed"].includes(job.state) || job.stats.finished === true);
  const left = $derived(eta(job));
</script>

<li class="flex flex-col gap-3 p-4" id={`download-${job.id}`}>
  <div class="flex flex-wrap items-start justify-between gap-3">
    <div class="min-w-0 flex-1"><h2 class="text-sm font-medium break-words">{label}</h2>
      <p class="mt-1 text-xs text-muted-foreground">{[job.title?.title, job.indexer_name, job.release_size ? bytes(job.release_size) : null].filter(Boolean).join(" · ")}</p>
      <p class="mt-1 font-mono text-[0.65rem] text-muted-foreground">Job {job.id.slice(0,8)}</p>
    </div><Badge variant={job.state === "error" ? "outline" : "secondary"} class="capitalize">{job.state}</Badge>
  </div>
  <p class="text-xs text-muted-foreground">{bytes(job.stats.progress_bytes ?? 0)} / {bytes(job.stats.total_bytes)} · {Math.round(progress(job))}%{left ? ` · ${left}` : ""} · Uploaded {bytes(job.stats.uploaded_bytes ?? 0)}</p>
  <div role="progressbar" aria-label={`Download progress for ${label}`} aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(progress(job))} class="h-1.5 overflow-hidden rounded-full bg-muted"><div class="h-full bg-primary transition-[width] duration-300" style:width={`${progress(job)}%`}></div></div>
  <div class="flex flex-wrap items-center justify-between gap-3">
    <span class="text-xs text-muted-foreground">↓ {bytes((job.stats.live?.download_speed.mbps ?? 0)*1024*1024)}/s · ↑ {bytes((job.stats.live?.upload_speed.mbps ?? 0)*1024*1024)}/s · {job.stats.live?.snapshot?.peer_stats?.live ?? 0} peers</span>
    <div class="flex flex-wrap gap-1">
      {#if ["paused","queued","completed"].includes(job.state)}<Button variant="ghost" size="icon-sm" aria-label="Resume transfer" disabled={pending} onclick={() => onaction("resume")}><PlayIcon /></Button>
      {:else if job.state === "error"}<Button variant="ghost" size="icon-sm" aria-label="Retry transfer" disabled={pending} onclick={() => onaction("retry")}><RotateCwIcon /></Button>
      {:else}<Button variant="ghost" size="icon-sm" aria-label="Pause transfer" disabled={pending} onclick={() => onaction("pause")}><PauseIcon /></Button>{/if}
      <Button variant="ghost" size="icon-sm" aria-label="Inspect files and selection" onclick={onfiles}><FilesIcon /></Button>
      {#if downloaded && session.user?.is_admin}<Button size="sm" variant="outline" onclick={onimport}>Review import</Button>{/if}
      <Button variant="ghost" size="icon-sm" aria-label="Remove job and keep files" onclick={onremove}><TrashIcon /></Button>
    </div>
  </div>
  <TransferLimit id={job.id} initial={job.download_limit_kib ?? 0} />
  {#if job.error}
    <p role="alert" class="text-sm text-destructive">{transferError(job.error)}</p>
    <details class="text-xs text-muted-foreground"><summary class="cursor-pointer">Technical details</summary><p class="mt-2 break-words">{job.error}</p></details>
  {/if}
  {#if downloaded}
    <p class="text-sm text-muted-foreground">{importMessage(job, importEnabled)}
      {#if !job.imported && session.user?.is_admin}
        {#if job.import_blocker && job.title_id}<a class="underline" href={href.wanted()}>Configure destination in Wanted</a>
        {:else}<a class="underline" href={href.settings("downloads/imports")}>Import settings</a>{/if}
      {/if}
    </p>
  {:else if job.import_warning}<p class="text-sm text-muted-foreground">Import review · {job.import_warning}</p>{/if}
  {#if job.queue_reason}<p class="text-xs text-muted-foreground">{job.queue_reason}</p>{/if}
</li>
