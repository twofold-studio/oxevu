<script lang="ts">
  import { onMount, untrack } from "svelte";
  import SearchIcon from "@lucide/svelte/icons/search";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import * as Field from "$lib/components/ui/field";
  import * as Empty from "$lib/components/ui/empty";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import PlateGrid from "$lib/features/library/PlateGrid.svelte";
  import TitlePlate from "$lib/features/library/TitlePlate.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { href } from "$lib/state/router.svelte";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import type { MediaKind } from "$lib/api/acquisition-types";
  import type { DiscoverFilters as Filters } from "$lib/media/discover-filters";
  import DiscoverFilters from "./DiscoverFilters.svelte";

  let { query, kind, page, filters = {} }: { query: string; kind: MediaKind; page: number; filters?: Filters } = $props();
  let text = $state(untrack(() => query));
  let selected = $state<Filters>(untrack(() => ({...filters})));
  const result = createResource(() => acquisitionApi.discover(query, kind, page, filters));
  const genres = createResource(() => acquisitionApi.genres(kind));
  onMount(() => { void result.load(); void genres.load(); });
  function search(event: SubmitEvent) { event.preventDefault(); window.location.hash = href.discover(text.trim(), kind, 1, selected); }
</script>

<Page>
  <PageHeader title="Discover" description="Browse movies and series, then find a release to download." />
  <form onsubmit={search} class="mb-6 flex flex-col gap-4">
    <div class="flex flex-wrap items-end gap-3">
    <Field.Field class="min-w-48 flex-1">
      <Field.Label for="discover-query" class="sr-only">Find a title</Field.Label>
      <Input id="discover-query" bind:value={text} placeholder="Search movies and series" maxlength={200} />
    </Field.Field>
    <Button type="submit"><SearchIcon data-icon="inline-start" />{text.trim() ? "Search" : "Apply filters"}</Button>
    <ToggleGroup.Root type="single" variant="outline" value={kind} onValueChange={(value) => value && (window.location.hash = href.discover(text, value as MediaKind, 1, {...selected, genre: undefined}))} aria-label="Content type">
      <ToggleGroup.Item value="movie">Movies</ToggleGroup.Item><ToggleGroup.Item value="tv">Series</ToggleGroup.Item>
    </ToggleGroup.Root>
    </div>
    <DiscoverFilters bind:value={selected} genres={genres.value?.genres ?? []} searching={Boolean(text.trim())} />
    {#if genres.error}<p class="text-xs text-muted-foreground">Genres are unavailable. Other filters still work. <button type="button" class="underline" onclick={() => genres.load()}>Retry genres</button></p>{/if}
    <div><Button variant="ghost" size="sm" href={href.discover("", kind)}>Clear search and filters</Button></div>
  </form>
  <ResourceGate status={result.status} error={result.error} what="discovery catalog" onretry={result.load}>
    {#if result.value?.results.length === 0}
      <Empty.Root class="border"><Empty.Header><Empty.Media variant="icon"><SearchIcon /></Empty.Media><Empty.Title>No matching titles</Empty.Title><Empty.Description>Try a different title, change the filters, or clear them to browse again.</Empty.Description></Empty.Header></Empty.Root>
    {:else}
      <PlateGrid label="Discover titles">
        {#each result.value?.results ?? [] as title (title.id)}
          <TitlePlate href={href.discoverTitle(title.kind, title.provider_id)} title={title.title} poster={title.poster_url} detail={title.year ?? "Year unknown"}>
              {#snippet badges()}
                {#if title.availability?.items.length}<Badge variant="secondary">In library</Badge>{:else if title.availability?.offline}<Badge variant="outline">Offline</Badge>{:else if title.availability?.requested}<Badge variant="outline">Requested</Badge>{/if}
              {/snippet}
          </TitlePlate>
        {/each}
      </PlateGrid>
    {/if}
      <div class="mt-8 flex items-center justify-between gap-3">
        <Button variant="outline" href={href.discover(query, kind, Math.max(1, page - 1), filters)} disabled={page <= 1}>Previous</Button>
        <span class="text-sm text-muted-foreground">Page {page}</span>
        <Button variant="outline" href={href.discover(query, kind, page + 1, filters)} disabled={page >= Math.min(20, result.value?.total_pages ?? page)}>Next</Button>
      </div>
      <p class="mt-5 text-xs text-muted-foreground">{result.value?.attribution}</p>
  </ResourceGate>
</Page>
