<script lang="ts">
  import { untrack } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { errorMessage } from "$lib/api/client";
  import type { Download, DiscoverTitle, MediaKind } from "$lib/api/acquisition-types";
  import { cn } from "$lib/utils";
  import Artwork from "$lib/components/Artwork.svelte";

  let { open = $bindable(), job, onidentified }: {
    open: boolean; job: Download; onidentified: () => void;
  } = $props();
  let query = $state(untrack(() => job.title?.title ?? job.release_title ?? job.files[0]?.path ?? ""));
  let kind = $state<MediaKind>("movie");
  let results = $state<DiscoverTitle[]>([]);
  let selected = $state<DiscoverTitle | null>(null);
  let searched = $state(false);
  let pending = $state(false);
  let failure = $state<string | null>(null);

  async function search(event: SubmitEvent) {
    event.preventDefault();
    if (!query.trim() || pending) return;
    pending = true;
    failure = null;
    selected = null;
    try {
      results = (await acquisitionApi.discover(query.trim(), kind)).results;
      searched = true;
    } catch (error) {
      results = [];
      failure = errorMessage(error);
    } finally {
      pending = false;
    }
  }

  async function confirm() {
    if (!selected || pending) return;
    pending = true;
    failure = null;
    try {
      await acquisitionApi.identify(job.id, selected.kind, selected.provider_id);
      onidentified();
      open = false;
    } catch (error) {
      failure = errorMessage(error);
    } finally {
      pending = false;
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content size="wide">
    <Dialog.Header>
      <Dialog.Title>Identify download</Dialog.Title>
      <Dialog.Description>Search for the correct title so the download can be imported into the right library.</Dialog.Description>
    </Dialog.Header>
    <form onsubmit={search}>
      <Field.Group>
        <Field.Field>
          <Field.Label for="dl-identify-title">Title</Field.Label>
          <div class="flex gap-2">
            <Input id="dl-identify-title" bind:value={query} disabled={pending} />
            <Button type="submit" disabled={pending || !query.trim()}>Search</Button>
          </div>
        </Field.Field>
        <Field.Field>
          <Field.Label>Type</Field.Label>
          <ToggleGroup.Root type="single" variant="outline" value={kind} disabled={pending}
            onValueChange={(value) => { if (value === "movie" || value === "tv") { kind = value; results = []; selected = null; searched = false; } }} aria-label="Media type">
            <ToggleGroup.Item value="movie">Movie</ToggleGroup.Item>
            <ToggleGroup.Item value="tv">Series</ToggleGroup.Item>
          </ToggleGroup.Root>
        </Field.Field>
      </Field.Group>
    </form>
    {#if pending}<p class="flex items-center gap-2 text-sm text-muted-foreground" role="status"><Spinner class="size-4" />Searching…</p>{/if}
    {#if failure}<p class="text-sm text-destructive" role="alert">{failure}</p>{/if}
    {#if results.length}
      <ul class="flex flex-col divide-y rounded-lg border" aria-label="Matching titles">
        {#each results as result (result.id)}
          <li>
            <button type="button" class={cn("flex w-full items-start gap-4 px-4 py-3 text-left transition-colors hover:bg-accent focus-visible:outline-2 focus-visible:outline-ring disabled:opacity-50", selected?.id === result.id && "bg-accent")}
              aria-pressed={selected?.id === result.id}
              disabled={pending} onclick={() => (selected = result)}>
              <Artwork src={result.poster_url} alt="" class="aspect-[2/3] w-14 shrink-0 rounded-md" />
              <span class="flex min-w-0 flex-1 flex-col gap-1">
                <span class="font-medium">{result.title}{result.year ? ` (${result.year})` : ""}</span>
                <span class="text-xs text-muted-foreground">{result.kind === "tv" ? "Series" : "Movie"}</span>
                {#if result.overview}<span class="line-clamp-3 text-sm text-muted-foreground">{result.overview}</span>{/if}
              </span>
            </button>
          </li>
        {/each}
      </ul>
    {:else if searched && !pending && !failure}
      <p class="text-sm text-muted-foreground">No results. Try a shorter or different title.</p>
    {/if}
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Close</Button>
      <Button disabled={!selected || pending} onclick={confirm}>Use this title</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
