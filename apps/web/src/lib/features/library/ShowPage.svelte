<script lang="ts">
  import PlayIcon from "@lucide/svelte/icons/play";
  import * as Tabs from "$lib/components/ui/tabs";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import Page from "$lib/components/Page.svelte";
  import CatalogGate from "$lib/components/CatalogGate.svelte";
  import NotFound from "$lib/components/NotFound.svelte";
  import DetailLayout from "$lib/components/DetailLayout.svelte";
  import { catalog } from "$lib/state/catalog.svelte";
  import { href, router } from "$lib/state/router.svelte";
  import { episodeCode, seasonLabel, type Season } from "$lib/media/grouping";
  import { plural } from "$lib/media/format";
  import { resumePosition } from "$lib/media/progress";
  import { artworkSrc } from "$lib/api/artwork";
  import EpisodeList from "./EpisodeList.svelte";
  import NextUpCard from "./NextUpCard.svelte";
  import PosterArt from "./PosterArt.svelte";

  /** URL segment: a slug, or a legacy internal key from older links. */
  let { showRef }: { showRef: string } = $props();

  const show = $derived(catalog.findShow(showRef));
  const seasonValue = (season: Season) => String(season.number ?? "none");

  // Legacy key links (and hand-typed titles) settle on the canonical slug URL
  // without adding a history entry.
  $effect(() => {
    if (show && show.slug !== showRef) router.go(href.show(show.slug), true);
  });

  // What "Play" means for a series: the episode in progress, else the next unwatched.
  const target = $derived(show ? (show.inProgress[0] ?? show.next) : undefined);
  const resuming = $derived(target ? resumePosition(target) !== null : false);
  const watched = $derived(show ? show.seasons.reduce((n, s) => n + s.episodes.filter((e) => e.completed).length, 0) : 0);

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
      <div class="grid gap-8 lg:grid-cols-[clamp(260px,24vw,340px)_minmax(0,1fr)] lg:gap-12">
        <Skeleton class="aspect-[2/3] w-40 rounded-xl sm:w-52 lg:w-full" />
        <div class="flex flex-col gap-4">
          <Skeleton class="h-10 w-2/3" />
          <Skeleton class="h-4 w-40" />
          <Skeleton class="mt-4 aspect-video w-full max-w-[28rem] rounded-xl" />
        </div>
      </div>
    </Page>
  {/snippet}

  {#if !show}
    <NotFound title="Series not found" description="It may have been removed by a rescan." />
  {:else}
    {#snippet posterArt()}
      <PosterArt src={show.posterUrl} title={show.title} />
    {/snippet}
    <DetailLayout
      backdrop={show.backdropUrl}
      poster={show.posterUrl}
      art={artworkSrc(show.posterUrl) ? posterArt : undefined}
    >

      <header class="flex flex-col gap-5">
        <div class="flex flex-col gap-2">
          <h1
            class="text-4xl leading-[1.05] font-semibold tracking-tight text-balance break-words [text-shadow:0_2px_24px_rgb(0_0_0/0.6)] lg:text-5xl"
          >
            {show.title}
          </h1>
          <p class="flex flex-wrap gap-x-4 gap-y-1 text-sm text-muted-foreground">
            <span>{plural(show.seasons.length, "season")}</span>
            <span>{plural(show.episodeCount, "episode")}</span>
            {#if watched > 0}
              <span>{watched === show.episodeCount ? "All watched" : `${watched} watched`}</span>
            {/if}
          </p>
        </div>
        {#if target}
          <Button href={href.watch(target.id)} size="lg" class="self-start">
            <PlayIcon data-icon="inline-start" />
            {resuming ? "Resume" : "Play"}
            {episodeCode(target) ?? target.title}
          </Button>
        {/if}
      </header>

      {#if target}
        <section aria-labelledby="next-up" class="flex flex-col gap-3">
          <h2 id="next-up" class="text-base font-medium">{resuming ? "Continue watching" : "Next up"}</h2>
          <NextUpCard item={target} />
        </section>
      {/if}

      <section aria-labelledby="episodes" class="flex flex-col gap-3">
        <h2 id="episodes" class="text-base font-medium">
          {show.seasons.length === 1 ? seasonLabel(show.seasons[0].number) : "Seasons"}
        </h2>
        {#if show.seasons.length === 1}
          <EpisodeList episodes={show.seasons[0].episodes} label={seasonLabel(show.seasons[0].number)} />
        {:else}
          <Tabs.Root value={activeSeason} onValueChange={(value) => (chosenSeason = value)}>
            <div class="mb-1 overflow-x-auto border-b">
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
      </section>
    </DetailLayout>
  {/if}
</CatalogGate>
