<script lang="ts">
  import { onDestroy } from "svelte";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import CheckIcon from "@lucide/svelte/icons/check";
  import * as Field from "$lib/components/ui/field";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Spinner } from "$lib/components/ui/spinner";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { Download, Release } from "$lib/api/acquisition-types";
  import { errorMessage } from "$lib/api/client";
  import { href } from "$lib/state/router.svelte";
  import { bytes } from "./format";
  import { downloadFailure } from "./release-search";

  let { release, profile, disabled = false }: { release: Release; profile: string; disabled?: boolean } = $props();
  let reviewing = $state(false);
  let override = $state(false);
  let pending = $state(false);
  let failure = $state<string | null>(null);
  let job = $state<Download | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let disposed = false;
  onDestroy(() => { disposed = true; clearTimeout(timer); });

  function observe(download: Download, attempts = 5) {
    if (disposed) return;
    job = download;
    failure = downloadFailure(download);
    // Catch startup failures from the asynchronous worker without polling forever.
    if (attempts && ["queued", "resolving", "checking"].includes(download.state)) {
      timer = setTimeout(() => {
        void acquisitionApi.downloadStatus(download.id).then(({ download }) => observe(download, attempts - 1)).catch(() => undefined);
      }, 1500);
    }
  }
  async function download() {
    if (pending || disabled) return;
    if (!release.accepted && !reviewing) { reviewing = true; return; }
    if (!release.accepted && !override) return;
    pending = true; failure = null; clearTimeout(timer);
    try {
      const result = job?.state === "error"
        ? await acquisitionApi.control(job.id, "retry")
        : await acquisitionApi.download({ release_id: release.id, profile_id: profile, override_profile: override, only_wanted: false });
      observe(result.download);
    } catch (error) { failure = errorMessage(error); }
    finally { pending = false; }
  }
</script>

<li class="flex flex-col gap-3 p-4">
  <div class="flex flex-wrap items-start justify-between gap-3">
    <div class="min-w-0 flex-1">
      <p class="text-sm font-medium break-words" title={release.title}>{release.title}</p>
      <p class="mt-1 text-xs leading-5 text-muted-foreground">{release.indexer_name} · {bytes(release.size)} · {release.seeders ?? "Unknown"} seeders · {release.height ? `${release.height}p` : "Resolution unknown"} · {release.codec ?? "Codec unknown"} · {release.source ?? "Source unknown"}</p>
      {#if release.also_on?.length}<p class="mt-1 text-xs text-muted-foreground">Also on {release.also_on.map((source) => source.indexer_name).join(", ")}</p>{/if}
    </div>
    <Badge variant={release.accepted ? "secondary" : "outline"}>{release.accepted ? "Meets profile" : "Review"}</Badge>
  </div>
  {#if !release.identity_match}<p class="text-xs text-muted-foreground">Title identity needs your review.</p>{/if}
  <div class="flex flex-wrap gap-1.5">{#each release.reasons as reason, index (`${index}:${reason}`)}<Badge variant="outline" class="whitespace-normal">{reason}</Badge>{/each}</div>
  {#if reviewing && !release.accepted}
    <Field.Field orientation="horizontal"><Checkbox id={`override-${release.id}`} bind:checked={override} /><Field.Label for={`override-${release.id}`}>Use this release despite the profile warnings</Field.Label></Field.Field>
  {/if}
  {#if failure}<p role="alert" class="text-sm text-destructive">{failure}</p>{/if}
  <div class="flex flex-wrap items-center justify-end gap-3">
    {#if job && !failure}
      <p role="status" class="flex items-center gap-1.5 text-xs text-muted-foreground"><CheckIcon class="size-3.5" />{job.state === "queued" ? "Download queued" : `Transfer ${job.state}`}</p>
      <Button href={href.activity()} size="sm" variant="outline">Open activity</Button>
    {:else}
      <Button size="sm" variant="outline" disabled={disabled || pending || (reviewing && !release.accepted && !override)} onclick={download}>
        {#if pending}<Spinner data-icon="inline-start" />{:else}<DownloadIcon data-icon="inline-start" />{/if}
        {pending ? "Queuing…" : job?.state === "error" ? "Retry download" : reviewing && !release.accepted ? "Confirm download" : "Download"}
      </Button>
    {/if}
  </div>
</li>
