<script lang="ts">
  import PlayIcon from "@lucide/svelte/icons/play";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import Page from "$lib/components/Page.svelte";
  import CatalogGate from "$lib/components/CatalogGate.svelte";
  import NotFound from "$lib/components/NotFound.svelte";
  import DetailLayout from "$lib/components/DetailLayout.svelte";
  import { catalog } from "$lib/state/catalog.svelte";
  import { href, router } from "$lib/state/router.svelte";
  import { episodeName, initialSeason, seasonLabel } from "$lib/media/grouping";
  import { plural } from "$lib/media/format";
  import { resumePosition } from "$lib/media/progress";
  import { artworkSrc } from "$lib/api/artwork";
  import EpisodeList from "./EpisodeList.svelte";
  import NextUpCard from "./NextUpCard.svelte";
  import SeasonPicker from "./SeasonPicker.svelte";
  import PosterArt from "./PosterArt.svelte";

  /** URL segment: a slug, or a legacy internal key from older links. */
  let { showRef, season }: { showRef: string; season?: string } = $props();

  const show = $derived(catalog.findShow(showRef));

  // Legacy key links (and hand-typed titles) settle on the canonical slug URL
  // without adding a history entry.
  $effect(() => {
    if (show && show.slug !== showRef) router.go(href.show(show.slug, season), true);
  });

  // What "Play" means for a series: the episode in progress, else the next unwatched.
  const target = $derived(show ? (show.inProgress[0] ?? show.next) : undefined);
  const resuming = $derived(target ? resumePosition(target) !== null : false);
  const watched = $derived(show ? show.seasons.reduce((n, s) => n + s.episodes.filter((e) => e.completed).length, 0) : 0);

  // The selected season lives in the URL (replaced, not pushed), so leaving
  // for an episode and pressing Back reopens the same tab. Without one, open
  // on the season holding the target episode.
  const activeSeason = $derived(show ? initialSeason(show, season, target) : "");
  function chooseSeason(value: string) {
    if (show && value !== activeSeason) router.go(href.show(show.slug, value), true);
  }
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
            {episodeName(target)}
          </Button>
        {/if}
      </header>

      {#if target}
        <section aria-labelledby="next-up" class="flex flex-col gap-3">
          <h2 id="next-up" class="text-base font-medium">{resuming ? "Continue watching" : "Next up"}</h2>
          <NextUpCard item={target} />
        </section>
      {/if}

      <section aria-labelledby="episodes" class="flex flex-col gap-4">
        <h2 id="episodes" class="text-base font-medium">
          {show.seasons.length === 1 ? seasonLabel(show.seasons[0].number) : "Seasons"}
        </h2>
        {#if show.seasons.length === 1}
          <EpisodeList
            episodes={show.seasons[0].episodes}
            label={seasonLabel(show.seasons[0].number)}
            upNextId={target?.id}
          />
        {:else}
          <SeasonPicker seasons={show.seasons} value={activeSeason} onValueChange={chooseSeason} upNextId={target?.id} />
        {/if}
      </section>
    </DetailLayout>
  {/if}
</CatalogGate>
