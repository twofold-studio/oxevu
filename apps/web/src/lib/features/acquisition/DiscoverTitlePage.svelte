<script lang="ts">
  import { onMount, untrack } from "svelte";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import FilmIcon from "@lucide/svelte/icons/film";
  import StarIcon from "@lucide/svelte/icons/star";
  import CalendarIcon from "@lucide/svelte/icons/calendar";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import Page from "$lib/components/Page.svelte";
  import Artwork from "$lib/components/Artwork.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { MediaKind, MonitoredTitle } from "$lib/api/acquisition-types";
  import { session } from "$lib/state/session.svelte";
  import { href } from "$lib/state/router.svelte";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import RequestDialog from "./RequestDialog.svelte";
  import ReleaseDialog from "./ReleaseDialog.svelte";
  import DiscoverTitleSkeleton from "./DiscoverTitleSkeleton.svelte";
  import SeriesSelectionDialog from "./SeriesSelectionDialog.svelte";
  import { newSeriesSelection } from "./series-selection";
  import { errorMessage } from "$lib/api/client";
  import { Skeleton } from "$lib/components/ui/skeleton";
  let { kind, id }: { kind: MediaKind; id: number } = $props();
  const resource = createResource(() => acquisitionApi.title(kind, id));
  let requestOpen = $state(false);let opening = $state(0);let sent = $state(false);
  onMount(() => { void resource.load(); });
  const canRequest = $derived(session.user?.is_admin || session.user?.acquisition?.can_request);
  const canDownload = $derived(session.user?.is_admin || session.user?.acquisition?.manage_downloads);
  let releaseOpen = $state(false);
  let releaseSeason = $state<number | undefined>(undefined);
  let releaseEpisode = $state<number | undefined>(undefined);
  let releaseOpening = $state(0);
  let monitorOpen = $state(false);
  let monitorEntry = $state<MonitoredTitle | null>(null);
  let monitorError = $state<string | null>(null);
  let monitorLoading = $state(false);
  let onlyWanted = $state(false);
  function find(season?: number, episode?: number, wanted = false) {
    releaseSeason = season;
    releaseEpisode = episode;
    onlyWanted = wanted;
    releaseOpening++;
    releaseOpen = true;
  }
  async function loadMonitor(open = false) {
    const title = resource.value?.title;
    if (!title || monitorLoading) return;
    monitorLoading = true;
    monitorError = null;
    try {
      const { titles } = await acquisitionApi.monitored();
      monitorEntry = titles.find((entry) => entry.title.id === title.id) ?? null;
      if (open && !monitorEntry) {
        monitorEntry = newSeriesSelection(title);
      }
      if (open) monitorOpen = true;
    } catch (error) {
      monitorError = errorMessage(error);
    } finally {
      monitorLoading = false;
    }
  }
  $effect(() => {
    if (resource.value && kind === "tv" && session.user?.is_admin) untrack(() => { void loadMonitor(); });
  });
  const hasSelection = $derived(monitorEntry?.monitor.enabled &&
    ["seasons", "episodes"].includes(monitorEntry.monitor.scope.mode));
</script>
<Page>
  <Button href={href.discover("", kind)} variant="ghost" class="mb-5"><ArrowLeftIcon data-icon="inline-start" />Discover</Button>
  <ResourceGate status={resource.status} error={resource.error} what="title details" onretry={resource.load}>
    {#snippet skeleton()}<DiscoverTitleSkeleton series={kind === "tv"} />{/snippet}
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
            {#if canDownload}<Button onclick={() => find()}>Find</Button>
            {:else if canRequest}<Button onclick={() => { opening++; requestOpen = true; }}>Request {kind === "tv" ? "episodes" : "movie"}</Button>{/if}
            {#if kind === "tv" && session.user?.is_admin}
              <Button variant="outline" disabled={monitorLoading} onclick={() => loadMonitor(true)}>Manage download selection</Button>
              {#if hasSelection}<Button variant="outline" onclick={() => find(undefined, undefined, true)}>Find selected episodes</Button>{/if}
            {/if}
            {#if title.availability?.items[0]}<Button variant="outline" href={href.item(title.availability.items[0].id)}>Open in library</Button>{/if}
          </div>
          {#if monitorLoading}<Skeleton class="h-4 w-44" aria-label="Loading download selection" />{/if}
          {#if monitorError}<p role="alert" class="text-xs text-destructive">{monitorError}</p>{/if}
          {#if sent}<p role="status" class="text-sm">Request submitted. You can follow it under Requests.</p>{/if}
          {#if title.availability?.offline}<p class="text-sm text-muted-foreground">Some library files are offline. Availability will update when the storage returns.</p>{/if}
          {#if kind === "tv"}
            <section class="rounded-xl border bg-card">
              <h2 class="border-b px-4 py-3 text-sm font-medium">Seasons</h2>
              <ul class="divide-y">
                {#each title.seasons ?? [] as season (season.season_number)}
                  <li class="px-4 py-3 text-sm">
                    <div class="flex items-center justify-between gap-3">
                      <span>{season.name}</span>
                      <span class="ml-auto text-muted-foreground">{season.episode_count} episodes</span>
                      {#if canDownload}<Button size="sm" variant="outline" aria-label={`Find ${season.name}`} onclick={() => find(season.season_number)}>Find season</Button>{/if}
                    </div>
                    {#if canDownload && resource.value.episodes.some((episode) => episode.season_number === season.season_number)}
                      <details class="mt-2">
                        <summary class="w-fit cursor-pointer text-muted-foreground">Browse episodes</summary>
                        <ul class="mt-2 divide-y border-t">
                          {#each resource.value.episodes.filter((episode) => episode.season_number === season.season_number) as episode (episode.id)}
                            <li class="flex items-center justify-between gap-3 py-2">
                              <span>{episode.episode_number}. {episode.name}</span>
                              <Button size="sm" variant="ghost" aria-label={`Find episode ${episode.episode_number}: ${episode.name}`} onclick={() => find(episode.season_number, episode.episode_number)}>Find episode</Button>
                            </li>
                          {/each}
                        </ul>
                      </details>
                    {/if}
                  </li>
                {/each}
              </ul>
            </section>
          {/if}
          <p class="text-xs text-muted-foreground">This product uses the TMDB API but is not endorsed or certified by TMDB.</p>
        </div>
      </div>
      {#key opening}<RequestDialog bind:open={requestOpen} {title} episodes={resource.value.episodes} onsent={() => { sent = true; void resource.load(); }} />{/key}
      {#if releaseOpen}{#key releaseOpening}<ReleaseDialog bind:open={releaseOpen} {title} season={releaseSeason} episode={releaseEpisode} {onlyWanted} autoSearch />{/key}{/if}
      {#if monitorOpen && monitorEntry}
        <SeriesSelectionDialog bind:open={monitorOpen} entry={monitorEntry} episodes={resource.value.episodes}
          onsaved={(monitor) => { if (monitorEntry) monitorEntry = { ...monitorEntry, monitor }; }} />
      {/if}
    {/if}
  </ResourceGate>
</Page>
