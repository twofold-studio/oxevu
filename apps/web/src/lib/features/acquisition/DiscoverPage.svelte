<script lang="ts">
  import { onMount } from "svelte";
  import SearchIcon from "@lucide/svelte/icons/search";
  import FilmIcon from "@lucide/svelte/icons/film";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import * as Field from "$lib/components/ui/field";
  import * as Empty from "$lib/components/ui/empty";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import Artwork from "$lib/components/Artwork.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { href } from "$lib/state/router.svelte";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import type { MediaKind } from "$lib/api/acquisition-types";

  let { query, kind, page }: { query: string; kind: MediaKind; page: number } = $props();
  let text = $state("");
  const result = createResource(() => acquisitionApi.discover(query, kind, page));
  onMount(() => { text = query; void result.load(); });
  function search(event: SubmitEvent) { event.preventDefault(); window.location.hash = href.discover(text.trim(), kind); }
</script>

<Page>
  <PageHeader title="Discover" description="Find a movie or series, check availability, and request what is missing." />
  <form onsubmit={search} class="mb-6 flex flex-wrap items-end gap-3">
    <Field.Field class="min-w-48 flex-1">
      <Field.Label for="discover-query" class="sr-only">Find a title</Field.Label>
      <Input id="discover-query" bind:value={text} placeholder="Search movies and series" maxlength={200} />
    </Field.Field>
    <Button type="submit"><SearchIcon data-icon="inline-start" />Search</Button>
    <ToggleGroup.Root type="single" variant="outline" value={kind} onValueChange={(value) => value && (window.location.hash = href.discover(text, value as MediaKind))} aria-label="Content type">
      <ToggleGroup.Item value="movie">Movies</ToggleGroup.Item><ToggleGroup.Item value="tv">Series</ToggleGroup.Item>
    </ToggleGroup.Root>
  </form>
  <ResourceGate status={result.status} error={result.error} what="discovery catalog" onretry={result.load}>
    {#if result.value?.results.length === 0}
      <Empty.Root class="border"><Empty.Header><Empty.Media variant="icon"><SearchIcon /></Empty.Media><Empty.Title>No matching titles</Empty.Title><Empty.Description>Try a different title or content type.</Empty.Description></Empty.Header></Empty.Root>
    {:else}
      <ul class="grid grid-cols-2 gap-x-4 gap-y-6 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
        {#each result.value?.results ?? [] as title (title.id)}
          <li>
            <a href={href.discoverTitle(title.kind, title.provider_id)} class="group flex flex-col gap-2 rounded-lg outline-offset-4 focus-visible:outline-2 focus-visible:outline-ring">
              <Artwork src={title.poster_url} alt="" class="aspect-[2/3] rounded-lg border transition-opacity group-hover:opacity-85">
                {#snippet fallback()}<div class="flex size-full items-center justify-center p-5"><FilmIcon class="size-10 text-muted-foreground" /></div>{/snippet}
              </Artwork>
              <span class="font-medium leading-snug">{title.title}</span>
              <span class="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">{title.year ?? "Year unknown"}
                {#if title.availability?.items.length}<Badge variant="secondary">In library</Badge>{:else if title.availability?.offline}<Badge variant="outline">Offline</Badge>{:else if title.availability?.requested}<Badge variant="outline">Requested</Badge>{/if}
              </span>
            </a>
          </li>
        {/each}
      </ul>
      <div class="mt-8 flex items-center justify-between gap-3">
        <Button variant="outline" href={href.discover(query, kind, Math.max(1, page - 1))} disabled={page <= 1}>Previous</Button>
        <span class="text-sm text-muted-foreground">Page {page}</span>
        <Button variant="outline" href={href.discover(query, kind, page + 1)} disabled={page >= Math.min(20, result.value?.total_pages ?? page)}>Next</Button>
      </div>
      <p class="mt-5 text-xs text-muted-foreground">{result.value?.attribution}</p>
    {/if}
  </ResourceGate>
</Page>
