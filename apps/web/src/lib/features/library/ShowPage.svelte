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
  import { episodeName, seasonKey, seasonLabel } from "$lib/media/grouping";
  import { plural } from "$lib/media/format";
  import { resumePosition } from "$lib/media/progress";
  import { artworkSrc } from "$lib/api/artwork";
  import EpisodeList from "./EpisodeList.svelte";
  import NextUpCard from "./NextUpCard.svelte";
  import SeasonShelf from "./SeasonShelf.svelte";
  import IdentifyButton from "./IdentifyButton.svelte";
  import MetadataDetails from "./MetadataDetails.svelte";
  import PosterArt from "./PosterArt.svelte";
  import TitleFacts from "./metadata/TitleFacts.svelte";
  import MetaFact from "./metadata/MetaFact.svelte";
  import { seasonRating } from "$lib/media/metadata";
  import CheckIcon from "@lucide/svelte/icons/check";
  import LayersIcon from "@lucide/svelte/icons/layers";
  import TvIcon from "@lucide/svelte/icons/tv";

  /** URL segment: a slug, or a legacy internal key from older links. */
  let { showRef, season }: { showRef: string; season?: string } = $props();

  const show = $derived(catalog.findShow(showRef));
  const selectedSeason = $derived(show?.seasons.find((entry) => seasonKey(entry) === season));

  // Legacy key links (and hand-typed titles) settle on the canonical slug URL
  // without adding a history entry.
  $effect(() => {
    if (show && show.slug !== showRef) router.go(href.show(show.slug, season), true);
  });

  // What "Play" means for a series: the episode in progress, else the next unwatched.
  const target = $derived(selectedSeason
    ? (selectedSeason.episodes.find((entry) => entry.position_seconds > 0 && !entry.completed) ?? selectedSeason.episodes.find((entry) => !entry.completed) ?? selectedSeason.episodes[0])
    : show ? (show.inProgress[0] ?? show.next) : undefined);
  const resuming = $derived(target ? resumePosition(target) !== null : false);
  const watched = $derived(show ? show.seasons.reduce((n, s) => n + s.episodes.filter((e) => e.completed).length, 0) : 0);

  // The selected season lives in the URL (replaced, not pushed), so leaving
  // for an episode and pressing Back reopens the same tab. Without one, open
  // on the season holding the target episode.
  const poster = $derived(selectedSeason?.posterUrl ?? show?.posterUrl);</script>

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
    <!-- A single-season series lists its episodes inline instead of covers. -->
    {@const overview = !selectedSeason && show.seasons.length !== 1}
    {#snippet posterArt()}
      <PosterArt src={poster} title={show.title} />
    {/snippet}
    {#snippet nextUp()}
      {#if target}
        <section aria-labelledby="next-up" class="flex flex-col gap-3">
          <h2 id="next-up" class="text-base font-medium">{resuming ? "Continue watching" : "Next up"}</h2>
          <NextUpCard item={target} />
        </section>
      {/if}
    {/snippet}
    <!-- A series with several seasons keeps Continue watching and the season
         covers in the side column, so they share the first viewport with the
         title wherever the screen is wide enough. -->
    {#snippet seasonsAside()}
      <!-- Between lg and xl the column sits under the details; pairing the two
           sections side by side there keeps the seasons from dropping a full
           card height further down. -->
      <div class="flex flex-col gap-8 lg:grid lg:grid-cols-2 lg:items-start xl:flex xl:items-stretch xl:gap-6">
        {@render nextUp()}
        <section aria-labelledby="seasons" class="flex flex-col gap-4">
          <h2 id="seasons" class="text-base font-medium">Seasons</h2>
          <SeasonShelf
            slug={show.slug}
            seasons={show.seasons}
            class="grid-cols-2 sm:grid-cols-[repeat(auto-fill,minmax(8rem,10rem))] lg:grid-cols-2"
          />
        </section>
      </div>
    {/snippet}
    <DetailLayout
      backdrop={show.backdropUrl}
      {poster}
      art={artworkSrc(poster) ? posterArt : undefined}
      aside={overview ? seasonsAside : undefined}
    >
      <header class="flex flex-col gap-5">
        <div class="flex flex-col gap-2">
          <h1
            class="text-4xl leading-[1.05] font-semibold tracking-tight text-balance break-words [text-shadow:0_2px_24px_rgb(0_0_0/0.6)] lg:text-5xl"
          >
            {show.title}
          </h1>
          {#if selectedSeason}
            <a href={href.show(show.slug)} class="text-sm text-muted-foreground hover:underline">Back to series</a>
            <h2 class="text-xl font-medium">{seasonLabel(selectedSeason.number)}</h2>
          {/if}
          {#snippet counts()}
            {#if !selectedSeason}<MetaFact icon={LayersIcon}>{plural(show.seasons.length, "season")}</MetaFact>{/if}
            <MetaFact icon={TvIcon}>{plural(selectedSeason?.episodes.length ?? show.episodeCount, "episode")}</MetaFact>
            {#if watched > 0}
              <MetaFact icon={CheckIcon}>{watched === show.episodeCount ? "All watched" : `${watched} watched`}</MetaFact>
            {/if}
          {/snippet}
          {#if target}
            <!-- A season page scores the season; without one it shows no score
                 rather than the series score under a season heading. -->
            <TitleFacts item={target} series rating={selectedSeason ? seasonRating(selectedSeason.episodes[0]) : undefined} children={counts} />
          {:else}
            <ul aria-label="Details" class="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-muted-foreground">{@render counts()}</ul>
          {/if}
        </div>
        {#if target}
          <div class="flex flex-wrap gap-2">
            <Button href={href.watch(target.id)} size="lg" class="h-auto max-w-full self-start py-3 whitespace-normal">
              <PlayIcon data-icon="inline-start" />
              {resuming ? "Resume" : "Play"}
              {episodeName(target)}
            </Button>
            <IdentifyButton item={target} />
          </div>
        {/if}
      </header>

      {#if target}<MetadataDetails item={target} series />{/if}
      {#if selectedSeason}
        {@const metadata = selectedSeason.episodes[0]}
        {#if metadata?.season_overview}<p class="max-w-3xl text-sm leading-7 text-muted-foreground">{metadata.season_overview}</p>{/if}
      {/if}

      {#if !overview}
        {#if !selectedSeason}{@render nextUp()}{/if}
        <section aria-labelledby="episodes" class="flex flex-col gap-4">
          <h2 id="episodes" class="text-base font-medium">
            {selectedSeason ? "Episodes" : seasonLabel(show.seasons[0].number)}
          </h2>
          {#if selectedSeason}
            <EpisodeList episodes={selectedSeason.episodes} label={seasonLabel(selectedSeason.number)} upNextId={target?.id} />
          {:else}
            <EpisodeList
              episodes={show.seasons[0].episodes}
              label={seasonLabel(show.seasons[0].number)}
              upNextId={target?.id}
            />
          {/if}
        </section>
      {/if}
    </DetailLayout>
  {/if}
</CatalogGate>
