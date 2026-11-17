<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import { metadataApi, type MetadataProvider, type MetadataResult } from "$lib/api/metadata";
  import { errorMessage } from "$lib/api/client";
  import { metadataId } from "$lib/api/metadata-id";
  import { identificationQuery } from "$lib/media/metadata";
  import type { Item } from "$lib/api/types";
  import { catalog } from "$lib/state/catalog.svelte";
  import { cn } from "$lib/utils";
  import { href } from "$lib/state/router.svelte";
  import MatchPoster from "./MatchPoster.svelte";

  let { open = $bindable(false), item, onsaved }: {
    open: boolean;
    item: Item;
    onsaved?: (item: Item) => void;
  } = $props();
  // The parent remounts this dialog for each opening.
  // svelte-ignore state_referenced_locally
  let query = $state(identificationQuery(item));
  let provider = $state<MetadataProvider>("tmdb");
  let results = $state<MetadataResult[]>([]);
  let selected = $state<MetadataResult | null>(null);
  let searched = $state(false);
  let pending = $state(false);
  let failure = $state<string | null>(null);
  let manualId = $state("");

  async function search(event: SubmitEvent) {
    event.preventDefault();
    if (!query.trim() || pending) return;
    pending = true;
    failure = null;
    selected = null;
    manualId = "";
    try {
      results = (await metadataApi.search(item.id, provider, query.trim())).results;
      searched = true;
    } catch (error) {
      results = [];
      failure = errorMessage(error);
    } finally {
      pending = false;
    }
  }

  async function identify() {
    if ((!selected && !manualId.trim()) || pending) return;
    pending = true;
    failure = null;
    try {
      const match = manualId.trim() ? { provider, id: metadataId(provider, manualId, item.kind === "episode") } : selected;
      if (!match) return;
      const { item: updated } = await metadataApi.match(item.id, match);
      catalog.patch(updated.id, updated);
      onsaved?.(updated);
      await catalog.ensure(true);
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
      <Dialog.Title>Identify {item.kind === "episode" ? "series" : "movie"}</Dialog.Title>
      <Dialog.Description>Find the correct title, then confirm its metadata. Media files stay unchanged.</Dialog.Description>
    </Dialog.Header>
    <form onsubmit={search}>
      <Field.Group>
        <Field.Field>
          <Field.Label for="identify-title">Title</Field.Label>
          <div class="flex gap-2">
            <Input id="identify-title" bind:value={query} disabled={pending} />
            <Button type="submit" disabled={pending || !query.trim()}>Search</Button>
          </div>
        </Field.Field>
        <Field.Field>
          <Field.Label>Provider</Field.Label>
          <ToggleGroup.Root type="single" variant="outline" value={provider} disabled={pending}
            onValueChange={(value) => { if (value === "tmdb" || value === "omdb") { provider = value; results = []; selected = null; manualId = ""; searched = false; } }} aria-label="Metadata provider">
            <ToggleGroup.Item value="tmdb">TMDB</ToggleGroup.Item>
            <ToggleGroup.Item value="omdb">OMDb</ToggleGroup.Item>
          </ToggleGroup.Root>
          <Field.Description>Configure provider credentials in <a class="underline" href={href.settings("media/metadata")}>Settings → Metadata</a>.</Field.Description>
        </Field.Field>
      </Field.Group>
    </form>
    <Field.Field>
      <Field.Label for="identify-provider-id">Or identify by {provider === "tmdb" ? "TMDB" : "IMDb"} ID</Field.Label>
      <Input id="identify-provider-id" bind:value={manualId} disabled={pending} placeholder={provider === "tmdb" ? "Numeric ID or TMDB title URL" : "tt1234567 or IMDb title URL"} />
      <Field.Description>
        <a class="underline" target="_blank" rel="noopener noreferrer" href={provider === "tmdb" ? `https://www.themoviedb.org/search?query=${encodeURIComponent(query)}` : `https://www.imdb.com/find/?q=${encodeURIComponent(query)}`}>Find this title on {provider === "tmdb" ? "TMDB" : "IMDb"}</a>, then paste its ID or URL here.{provider === "omdb" ? " OMDb looks up titles using IMDb IDs." : ""}
      </Field.Description>
    </Field.Field>
    {#if pending}<p class="flex items-center gap-2 text-sm text-muted-foreground" role="status"><Spinner class="size-4" />Loading…</p>{/if}
    {#if failure}<p class="text-sm text-destructive" role="alert">{failure}</p>{/if}
    {#if results.length}
      <ul class="flex flex-col divide-y rounded-lg border" aria-label="Matching titles">
        {#each results as result (`${result.provider}:${result.id}`)}
          <li>
            <button type="button" class={cn("flex w-full items-start gap-4 px-4 py-3 text-left transition-colors hover:bg-accent focus-visible:outline-2 focus-visible:outline-ring disabled:opacity-50", selected?.id === result.id && selected.provider === result.provider && "bg-accent")}
              aria-pressed={selected?.id === result.id && selected.provider === result.provider}
              disabled={pending} onclick={() => { selected = result; manualId = ""; }}>
              <MatchPoster url={result.poster_url} />
              <span class="flex min-w-0 flex-1 flex-col gap-1">
              <span class="font-medium">{result.title}{result.year ? ` (${result.year})` : ""}</span>
              <span class="text-xs text-muted-foreground">{result.kind === "tv" ? "Series" : "Movie"} · {result.provider === "tmdb" ? "TMDB" : "OMDb"}</span>
              {#if result.overview}<span class="line-clamp-3 text-sm text-muted-foreground">{result.overview}</span>{/if}
              </span>
            </button>
          </li>
        {/each}
      </ul>
    {:else if searched && !pending && !failure}
      <p class="text-sm text-muted-foreground">No matching titles. Try a shorter title or another provider.</p>
    {/if}
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Close</Button>
      <Button disabled={(!selected && !manualId.trim()) || pending} onclick={identify}>{manualId.trim() ? "Identify by ID" : "Use this match"}</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
