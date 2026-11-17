<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { href } from "$lib/state/router.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import type { MediaKind } from "$lib/api/acquisition-types";
  import type { DiscoverFilters as Filters } from "$lib/media/discover-filters";
  import DiscoverFilters from "./DiscoverFilters.svelte";
  import DiscoverResults from "./DiscoverResults.svelte";

  let { query, kind, page, filters = {} }: { query: string; kind: MediaKind; page: number; filters?: Filters } = $props();
  let text = $state(untrack(() => query));
  let selected = $state<Filters>(untrack(() => ({...filters})));
  const genres = createResource(() => acquisitionApi.genres(kind));
  let debounce: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    clearTimeout(debounce);
    text = query;
    selected = { ...filters };
  });
  $effect(() => {
    void kind;
    untrack(() => { void genres.load(); });
  });
  onDestroy(() => clearTimeout(debounce));

  function apply(nextKind = kind, nextFilters = selected) {
    clearTimeout(debounce);
    const target = href.discover(text.trim(), nextKind, 1, nextFilters);
    if (target !== window.location.hash) window.location.hash = target;
  }

  function typeQuery(event: Event & { currentTarget: HTMLInputElement }) {
    text = event.currentTarget.value;
    clearTimeout(debounce);
    debounce = setTimeout(() => apply(), 300);
  }
</script>

<Page>
  <PageHeader title="Discover" description="Browse movies and series, then find a release to download." />
  <form onsubmit={(event) => { event.preventDefault(); apply(); }} class="mb-6 flex flex-col gap-4">
    <div class="flex flex-wrap items-end gap-3">
    <Field.Field class="min-w-48 flex-1">
      <Field.Label for="discover-query" class="sr-only">Find a title</Field.Label>
      <Input id="discover-query" value={text} oninput={typeQuery} placeholder="Search movies and series" maxlength={200} />
    </Field.Field>
    <ToggleGroup.Root type="single" variant="outline" value={kind} onValueChange={(value) => value && apply(value as MediaKind, {...selected, genre: undefined})} aria-label="Content type">
      <ToggleGroup.Item value="movie">Movies</ToggleGroup.Item><ToggleGroup.Item value="tv">Series</ToggleGroup.Item>
    </ToggleGroup.Root>
    </div>
    <DiscoverFilters value={selected} onchange={(next) => { selected = next; apply(kind, next); }} genres={genres.value?.genres ?? []} searching={Boolean(text.trim())} />
    {#if genres.error}<p class="text-xs text-muted-foreground">Genres are unavailable. Other filters still work. <button type="button" class="underline" onclick={() => genres.load()}>Retry genres</button></p>{/if}
    <div><Button variant="ghost" size="sm" onclick={() => { text = ""; selected = {}; apply(); }}>Clear search and filters</Button></div>
  </form>
  {#key href.discover(query, kind, page, filters)}
    <DiscoverResults {query} {kind} {page} {filters} />
  {/key}
</Page>
