<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";
  import { Spinner } from "$lib/components/ui/spinner";
  import { Button } from "$lib/components/ui/button";
  import * as Alert from "$lib/components/ui/alert";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";
  import { removalApi, type RemovalPreview, type RemovalResult } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import { catalog } from "$lib/state/catalog.svelte";
  import { plural } from "$lib/media/format";

  let {
    open = $bindable(false),
    itemIds,
    label,
    onremoved,
  }: {
    open: boolean;
    /** Explicit IDs of items to remove (movies: item IDs; shows/seasons: episode IDs). */
    itemIds: string[];
    /** Human label for the dialog title, e.g. "Breaking Bad" or "3 movies". */
    label: string;
    onremoved?: (result: RemovalResult) => void;
  } = $props();

  type Phase = "options" | "preview" | "confirming" | "done" | "error";

  let phase = $state<Phase>("options");
  let keepMetadata = $state(true);
  let preview = $state<RemovalPreview | null>(null);
  let result = $state<RemovalResult | null>(null);
  let acknowledged = $state(false);
  let failure = $state<string | null>(null);
  const localCount = $derived(preview?.items.filter((entry) => entry.local_file).length ?? 0);

  function reset() {
    phase = "options";
    keepMetadata = true;
    preview = null;
    result = null;
    acknowledged = false;
    failure = null;
  }

  $effect(() => {
    if (open) reset();
  });

  async function loadPreview() {
    phase = "preview";
    failure = null;
    try {
      preview = await removalApi.preview(itemIds, keepMetadata);
    } catch (error) {
      failure = errorMessage(error);
      phase = "error";
    }
  }

  async function confirmRemoval(event: Event) {
    event.preventDefault();
    if (!preview || !acknowledged || phase !== "preview" || (keepMetadata && !localCount)) return;
    phase = "confirming";
    failure = null;
    try {
      result = await removalApi.remove(itemIds, keepMetadata, preview.token);
      phase = "done";
      for (const id of result.removed_ids) catalog.patch(id, { file_available: false });
      try { await catalog.ensure(true); }
      catch { failure = "Removal completed, but the library could not refresh. Reload the page to see its latest state."; }
      onremoved?.(result);
    } catch (error) {
      failure = errorMessage(error);
      phase = "error";
    }
  }

  function formatBytes(bytes: number): string {
    if (!Number.isFinite(bytes) || bytes < 0) return "0 B";
    const unit = bytes >= 1024 ** 3 ? 3 : bytes >= 1024 ** 2 ? 2 : bytes >= 1024 ? 1 : 0;
    return `${(bytes / 1024 ** unit).toLocaleString(undefined, { maximumFractionDigits: 2 })} ${["B", "KiB", "MiB", "GiB"][unit]}`;
  }
</script>

<AlertDialog.Root bind:open={() => open, (value) => { if (phase !== "confirming") open = value; }}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Remove {label}?</AlertDialog.Title>
      <AlertDialog.Description>
        This permanently deletes the selected media files from disk. Artwork, subtitles and torrent
        sources are kept.
      </AlertDialog.Description>
    </AlertDialog.Header>

    {#if phase === "options"}
      <div class="flex flex-col gap-4">
        <div class="flex items-center gap-3">
          <Switch id="keep-metadata" bind:checked={keepMetadata} />
          <Label for="keep-metadata" class="text-sm">
            Keep metadata and history
          </Label>
        </div>
        <p class="text-xs text-muted-foreground">
          {#if keepMetadata}
            Titles stay browsable and marked <strong>No local file</strong>. Watch progress, favorites
            and opinions are kept for all users.
          {:else}
            All metadata, watch progress, favorites and opinions for these titles are deleted for
            <strong>every user</strong>. This cannot be undone.
          {/if}
        </p>
      </div>
      <AlertDialog.Footer>
        <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
        <AlertDialog.Action
          variant="destructive"
          onclick={(e: Event) => { e.preventDefault(); loadPreview(); }}
        >
          Preview removal
        </AlertDialog.Action>
      </AlertDialog.Footer>

    {:else if phase === "preview" && !preview}
      <div class="flex items-center justify-center py-8" aria-busy="true">
        <Spinner class="size-6" />
      </div>

    {:else if phase === "preview" && preview}
      <div class="flex flex-col gap-3">
        <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-sm">
          <dt class="text-muted-foreground">Local files</dt>
          <dd>{plural(localCount, "file")}</dd>
          <dt class="text-muted-foreground">Library entries</dt>
          <dd>{plural(preview.count, "entry", "entries")}</dd>
          <dt class="text-muted-foreground">Selected size</dt>
          <dd>{formatBytes(preview.bytes_selected)}</dd>
        </dl>
        {#if preview.shared_files_count > 0}
          <Alert.Root>
            <TriangleAlertIcon />
            <Alert.Description>
              {plural(preview.shared_files_count, "file")} {preview.shared_files_count === 1 ? "is" : "are"} hard-linked
              elsewhere. Deleting {preview.shared_files_count === 1 ? "it" : "them"} may not free disk space.
            </Alert.Description>
          </Alert.Root>
        {/if}
        {#if preview.notice}
          <p class="text-xs text-muted-foreground">{preview.notice}</p>
        {/if}
          <details class="text-sm">
            <summary class="cursor-pointer text-muted-foreground hover:underline">
              Show {plural(preview.items.length, "title")}
            </summary>
            <ul class="mt-1 max-h-48 list-inside list-disc overflow-y-auto text-xs text-muted-foreground">
              {#each preview.items as entry (entry.id)}
                <li>{entry.title}{entry.size_bytes != null ? ` (${formatBytes(entry.size_bytes)})` : ""}</li>
              {/each}
            </ul>
          </details>
        <label class="flex items-start gap-3 pt-2">
          <Checkbox bind:checked={acknowledged} />
          <span class="text-sm">
            I understand this permanently deletes
            {plural(localCount, "media file")} from disk.
            {#if !keepMetadata}Metadata and history for {plural(preview.count, "entry", "entries")} will also be forgotten for every user.{/if}
          </span>
        </label>
      </div>
      <AlertDialog.Footer>
        <Button variant="outline" onclick={() => { phase = "options"; preview = null; acknowledged = false; }}>Back</Button>
        <AlertDialog.Action
          variant="destructive"
          onclick={confirmRemoval}
          disabled={!acknowledged || (keepMetadata && !localCount)}
        >
          {localCount ? (keepMetadata ? "Delete files" : "Delete files and forget") : "Forget entries"}
        </AlertDialog.Action>
      </AlertDialog.Footer>

    {:else if phase === "confirming"}
      <div class="flex items-center justify-center gap-3 py-8" aria-busy="true">
        <Spinner class="size-6" />
        <span class="text-sm text-muted-foreground">Removing files…</span>
      </div>

    {:else if phase === "done" && result}
      <div class="flex flex-col gap-2 text-sm">
        <p>{plural(result.deleted_files ?? localCount, "file")} deleted; {plural(result.removed_ids.length, "entry", "entries")} updated.</p>
        <p class="text-xs text-muted-foreground">Estimated space reclaimed: {formatBytes(result.bytes_freed_estimate)}. Shared download copies or open files can keep storage allocated.</p>
        {#if failure}<p class="text-sm text-destructive" role="status">{failure}</p>{/if}
        {#if result.failures.length > 0}
          <Alert.Root variant="destructive">
            <Alert.Title>Some removal steps failed</Alert.Title>
            <Alert.Description>
              <ul class="list-inside list-disc text-xs">
                {#each result.failures as f (f.id)}<li>{f.message}</li>{/each}
              </ul>
            </Alert.Description>
          </Alert.Root>
        {/if}
      </div>
      <AlertDialog.Footer>
        <AlertDialog.Action onclick={() => (open = false)}>Close</AlertDialog.Action>
      </AlertDialog.Footer>

    {:else if phase === "error"}
      <p class="text-sm text-destructive" role="alert">{failure}</p>
      <AlertDialog.Footer>
        <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
        <AlertDialog.Action variant="destructive" onclick={(e: Event) => { e.preventDefault(); loadPreview(); }}>
          Retry
        </AlertDialog.Action>
      </AlertDialog.Footer>
    {/if}
  </AlertDialog.Content>
</AlertDialog.Root>
