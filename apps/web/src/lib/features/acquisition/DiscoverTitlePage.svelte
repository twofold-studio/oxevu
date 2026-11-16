<script lang="ts">
  import { onMount } from "svelte";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import FilmIcon from "@lucide/svelte/icons/film";
  import StarIcon from "@lucide/svelte/icons/star";
  import CalendarIcon from "@lucide/svelte/icons/calendar";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import Page from "$lib/components/Page.svelte";
  import Artwork from "$lib/components/Artwork.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { MediaKind } from "$lib/api/acquisition-types";
  import { session } from "$lib/state/session.svelte";
  import { href } from "$lib/state/router.svelte";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import RequestDialog from "./RequestDialog.svelte";
  let { kind, id }: { kind: MediaKind; id: number } = $props();
  const resource = createResource(() => acquisitionApi.title(kind, id));
  let requestOpen = $state(false);let opening = $state(0);let sent = $state(false);
  onMount(() => { void resource.load(); });
  const canRequest = $derived(session.user?.is_admin || session.user?.acquisition?.can_request);
</script>
<Page>
  <Button href={href.discover("", kind)} variant="ghost" class="mb-5"><ArrowLeftIcon data-icon="inline-start" />Discover</Button>
  <ResourceGate status={resource.status} error={resource.error} what="title details" onretry={resource.load}>
    {#if resource.value}
      {@const title = resource.value.title}
      <div class="grid items-start gap-7 md:grid-cols-[minmax(12rem,20rem)_minmax(0,1fr)]">
        <Artwork src={title.poster_url} alt={`Poster for ${title.title}`} eager class="aspect-[2/3] w-full max-w-80 rounded-xl border">
          {#snippet fallback()}<div class="flex size-full items-center justify-center"><FilmIcon class="size-12 text-muted-foreground" /></div>{/snippet}
        </Artwork>
        <div class="flex min-w-0 flex-col gap-5">
          <h1 class="text-3xl font-semibold tracking-tight sm:text-4xl">{title.title}</h1>
          <div class="flex flex-wrap items-center gap-4 text-sm text-muted-foreground">
            {#if title.year}<span class="inline-flex items-center gap-1.5"><CalendarIcon class="size-4" />{title.year}</span>{/if}
            {#if title.rating}<span class="inline-flex items-center gap-1.5" aria-label={`TMDB rating ${title.rating} out of 10`}><StarIcon class="size-4 text-rating" />{title.rating.toFixed(1)}</span>{/if}
            <Badge variant="outline">{kind === "tv" ? "Series" : "Movie"}</Badge>
          </div>
          <p class="max-w-3xl text-sm leading-7 text-muted-foreground">{title.overview || "No description is available."}</p>
          <div class="flex flex-wrap gap-2">
            {#if canRequest}<Button onclick={() => { opening++; requestOpen = true; }}>Request {kind === "tv" ? "episodes" : "movie"}</Button>{/if}
            {#if title.availability?.items[0]}<Button variant="outline" href={href.item(title.availability.items[0].id)}>Open in library</Button>{/if}
            <Button href={href.requests()} variant="outline">Requests</Button>
          </div>
          {#if sent}<p role="status" class="text-sm">Request submitted. You can follow it under Requests.</p>{/if}
          {#if title.availability?.offline}<p class="text-sm text-muted-foreground">Some library files are offline. Availability will update when the storage returns.</p>{/if}
          {#if kind === "tv"}
            <section class="rounded-xl border bg-card">
              <h2 class="border-b px-4 py-3 text-sm font-medium">Seasons</h2>
              <ul class="divide-y">{#each title.seasons ?? [] as season (season.season_number)}<li class="flex justify-between gap-3 px-4 py-3 text-sm"><span>{season.name}</span><span class="text-muted-foreground">{season.episode_count} episodes</span></li>{/each}</ul>
            </section>
          {/if}
          <p class="text-xs text-muted-foreground">This product uses the TMDB API but is not endorsed or certified by TMDB.</p>
        </div>
      </div>
      {#key opening}<RequestDialog bind:open={requestOpen} {title} episodes={resource.value.episodes} onsent={() => { sent = true; void resource.load(); }} />{/key}
    {/if}
  </ResourceGate>
</Page>
