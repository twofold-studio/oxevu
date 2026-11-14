<script lang="ts">
  import PlayIcon from "@lucide/svelte/icons/play";
  import * as Tabs from "$lib/components/ui/tabs";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import Page from "$lib/components/Page.svelte";
  import CatalogGate from "$lib/components/CatalogGate.svelte";
  import NotFound from "$lib/components/NotFound.svelte";
  import { catalog } from "$lib/state/catalog.svelte";
  import { href } from "$lib/state/router.svelte";
  import { episodeCode, seasonLabel, type Season } from "$lib/media/grouping";
  import { plural } from "$lib/media/format";
  import { resumePosition } from "$lib/media/progress";
  import Artwork from "$lib/components/Artwork.svelte";
  import Backdrop from "$lib/components/Backdrop.svelte";
  import EpisodeList from "./EpisodeList.svelte";

  let { showKey }: { showKey: string } = $props();

  const show = $derived(catalog.shows.find((s) => s.key === showKey));
  const seasonValue = (season: Season) => String(season.number ?? "none");

  // What "Play" means for a series: the episode in progress, else the next unwatched.
  const target = $derived(show ? (show.inProgress[0] ?? show.next) : undefined);
  const resuming = $derived(target ? resumePosition(target) !== null : false);

  // Open on the season holding the target episode; user choice wins afterwards.
  let chosenSeason = $state<string | null>(null);
  const activeSeason = $derived(
    chosenSeason ??
      (show && target
        ? seasonValue(show.seasons.find((s) => s.episodes.includes(target)) ?? show.seasons[0])
        : ""),
  );
</script>

<CatalogGate>
  {#snippet skeleton()}
    <Page>
      <Skeleton class="mb-3 h-8 w-64" />
      <Skeleton class="mb-8 h-4 w-40" />
      <Skeleton class="h-72 w-full rounded-xl" />
    </Page>
  {/snippet}

  {#if !show}
    <NotFound title="Series not found" description="It may have been removed by a rescan." />
  {:else}
    <Page class="relative isolate">
      <Backdrop src={show.backdropUrl} />
      <div class="mb-8 flex flex-col gap-5 border-b pb-8 sm:flex-row sm:items-end sm:justify-between">
        <div class="flex min-w-0 items-end gap-5">
          <Artwork
            src={show.posterUrl}
            alt={`Poster for ${show.title}`}
            eager
            class="aspect-[2/3] w-24 shrink-0 rounded-lg border shadow-lg shadow-black/40 sm:w-36"
          />
          <div class="flex min-w-0 flex-col gap-2">
          <h1 class="text-3xl font-semibold tracking-tight text-balance sm:text-4xl">{show.title}</h1>
          <p class="text-sm text-muted-foreground">
            {plural(show.seasons.length, "season")}, {plural(show.episodeCount, "episode")}
          </p>
          </div>
        </div>
        {#if target}
          <Button href={href.watch(target.id)} size="lg" class="self-start sm:self-auto">
            <PlayIcon data-icon="inline-start" />
            {resuming ? "Resume" : "Play"}
            {episodeCode(target) ?? target.title}
          </Button>
        {/if}
      </div>

      {#if show.seasons.length === 1}
        <h2 class="mb-3 text-base font-medium">{seasonLabel(show.seasons[0].number)}</h2>
        <EpisodeList episodes={show.seasons[0].episodes} label={seasonLabel(show.seasons[0].number)} />
      {:else}
        <Tabs.Root value={activeSeason} onValueChange={(value) => (chosenSeason = value)}>
          <div class="mb-4 overflow-x-auto border-b">
            <Tabs.List variant="line">
              {#each show.seasons as season (seasonValue(season))}
                <Tabs.Trigger value={seasonValue(season)}>{seasonLabel(season.number)}</Tabs.Trigger>
              {/each}
            </Tabs.List>
          </div>
          {#each show.seasons as season (seasonValue(season))}
            <Tabs.Content value={seasonValue(season)}>
              <EpisodeList episodes={season.episodes} label={seasonLabel(season.number)} />
            </Tabs.Content>
          {/each}
        </Tabs.Root>
      {/if}
    </Page>
  {/if}
</CatalogGate>
