<script lang="ts">
  import FavoriteButton from "./FavoriteButton.svelte";
  import PlayIcon from "@lucide/svelte/icons/play";
  import TrashIcon from "@lucide/svelte/icons/trash-2";
  import CheckIcon from "@lucide/svelte/icons/check";
  import LayersIcon from "@lucide/svelte/icons/layers";
  import TvIcon from "@lucide/svelte/icons/tv";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import Page from "$lib/components/Page.svelte";
  import CatalogGate from "$lib/components/CatalogGate.svelte";
  import NotFound from "$lib/components/NotFound.svelte";
  import DetailLayout from "$lib/components/DetailLayout.svelte";
  import { catalog, itemPlayable } from "$lib/state/catalog.svelte";
  import { session } from "$lib/state/session.svelte";
  import { isAdmin, canManageDownloads } from "$lib/state/access";
  import RemovalDialog from "$lib/features/details/RemovalDialog.svelte";
  import FindAgainLink from "$lib/features/details/FindAgainLink.svelte";
  import type { RemovalResult } from "$lib/api/endpoints";
  import type { Item } from "$lib/api/types";
  import { href, router } from "$lib/state/router.svelte";
  import { episodeName, initialSeason, seasonLabel, type Season } from "$lib/media/grouping";
  import { plural } from "$lib/media/format";
  import { resumePosition } from "$lib/media/progress";
  import { artworkSrc } from "$lib/api/artwork";
  import NextUpCard from "./NextUpCard.svelte";
  import SeasonPicker from "./SeasonPicker.svelte";
  import IdentifyButton from "./IdentifyButton.svelte";
  import MetadataDetails from "./MetadataDetails.svelte";
  import PosterArt from "./PosterArt.svelte";
  import TitleFacts from "./metadata/TitleFacts.svelte";
  import MetaFact from "./metadata/MetaFact.svelte";

  // One page per series: a compact hero for the series (Play/Resume, series
  // actions, overview), then a full-width Episodes section whose season tabs
  // sit directly above the episode list. `season` (from the URL) only picks
  // the open tab; it is replaced, not pushed, so Back from an episode
  // reopens the same season.

  /** URL segment: a slug, or a legacy internal key from older links. */
  let { showRef, season }: { showRef: string; season?: string } = $props();

  const show = $derived(catalog.findShow(showRef));

  // Legacy key links (and hand-typed titles) settle on the canonical slug URL
  // without adding a history entry.
  $effect(() => {
    if (show && show.slug !== showRef) router.go(href.show(show.slug, season), true);
  });

  // What "Play" means for a series: the episode in progress, else the next unwatched.
  const episodes = $derived(show?.seasons.flatMap((s) => s.episodes) ?? []);
  const target = $derived(
    episodes.find((e) => e.position_seconds > 0 && !e.completed && itemPlayable(e)) ??
      episodes.find((e) => !e.completed && itemPlayable(e)) ??
      episodes.find(itemPlayable) ??
      episodes[0],
  );
  const targetPlayable = $derived(target ? itemPlayable(target) : false);
  const resuming = $derived(target ? resumePosition(target) !== null : false);
  const watched = $derived(episodes.filter((e) => e.completed).length);
  // A stale or hand-edited season key falls back to the season holding Play's target.
  const openSeason = $derived(show ? initialSeason(show, season, target) : "");

  const admin = $derived(isAdmin(session.user));
  const canDownload = $derived(canManageDownloads(session.user));

  // One dialog serves both the series and a single season.
  let removalOpen = $state(false);
  let removal = $state<{ ids: string[]; label: string; series: boolean }>({ ids: [], label: "", series: true });
  const transferring = (items: Item[]) => items.some((e) => !!e.download);
  function confirmRemoval(items: Item[], label: string, series: boolean) {
    removal = { ids: items.map((e) => e.id), label, series };
    removalOpen = true;
  }
  function handleRemoved(result: RemovalResult) {
    if (removal.series && !result.kept_metadata && !result.failures.length) router.go(href.home());
  }
</script>

<CatalogGate>
  {#snippet skeleton()}
    <Page>
      <div class="grid gap-8 lg:grid-cols-[clamp(220px,22vw,300px)_minmax(0,1fr)] lg:gap-10">
        <Skeleton class="aspect-[2/3] w-32 rounded-xl sm:w-44 lg:w-full" />
        <div class="flex flex-col gap-4">
          <Skeleton class="h-10 w-2/3" />
          <Skeleton class="h-4 w-40" />
          <Skeleton class="h-9 w-56" />
        </div>
      </div>
      <Skeleton class="mt-12 h-10 w-full" />
    </Page>
  {/snippet}

  {#if !show}
    <NotFound title="Series not found" description="It may have been removed by a rescan." />
  {:else}
    {#snippet posterArt()}
      <PosterArt src={show.posterUrl} title={show.title} />
    {/snippet}

    <!-- Wide screens only: the hero's Play button and the marked episode row
         already offer the same episode everywhere else. -->
    {#snippet nextUp()}
      {#if target && targetPlayable}
        <section aria-labelledby="next-up" class="flex flex-col gap-3">
          <h2 id="next-up" class="text-sm font-medium text-muted-foreground">{resuming ? "Continue watching" : "Next up"}</h2>
          <NextUpCard item={target} />
        </section>
      {/if}
    {/snippet}

    {#snippet seasonActions(entry: Season)}
      {@const label = seasonLabel(entry.number)}
      <FavoriteButton items={entry.episodes} label="Favorite season" size="sm" />
      {#if admin}
        <Button
          variant="outline"
          size="sm"
          disabled={transferring(entry.episodes)}
          title={transferring(entry.episodes) ? "Manage active transfers in Downloads before removing this season" : undefined}
          onclick={() => confirmRemoval(entry.episodes, `${show.title} · ${label}`, false)}
        >
          <TrashIcon data-icon="inline-start" />Remove season
        </Button>
      {/if}
    {/snippet}

    {#snippet episodeSection()}
      <section aria-labelledby="episodes" class="flex flex-col gap-4">
        <h2 id="episodes" class="text-xl font-semibold tracking-tight">{show.seasons.length > 1 ? "Seasons" : "Episodes"}</h2>
        <SeasonPicker
          seasons={show.seasons}
          value={openSeason}
          onValueChange={(key) => router.go(href.show(show.slug, key), true)}
          upNextId={target?.id}
          seriesPoster={show.posterUrl}
          actions={seasonActions}
        />
      </section>
    {/snippet}

    <DetailLayout
      backdrop={show.backdropUrl}
      poster={show.posterUrl}
      art={artworkSrc(show.posterUrl) ? posterArt : undefined}
      aside={target && targetPlayable ? nextUp : undefined}
      below={show.seasons.length ? episodeSection : undefined}
    >
      <header class="flex flex-col gap-5">
        <div class="flex flex-col gap-2">
          <h1
            class="text-3xl leading-[1.05] font-semibold tracking-tight text-balance break-words [text-shadow:0_2px_24px_rgb(0_0_0/0.6)] sm:text-4xl lg:text-5xl"
          >
            {show.title}
          </h1>
          {#snippet counts()}
            <MetaFact icon={LayersIcon}>{plural(show.seasons.length, "season")}</MetaFact>
            <MetaFact icon={TvIcon}>{plural(show.episodeCount, "episode")}</MetaFact>
            {#if watched > 0}
              <MetaFact icon={CheckIcon}>{watched === show.episodeCount ? "All watched" : `${watched} watched`}</MetaFact>
            {/if}
          {/snippet}
          {#if target}
            <TitleFacts item={target} series files={episodes} children={counts} />
          {:else}
            <ul aria-label="Details" class="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-muted-foreground">{@render counts()}</ul>
          {/if}
        </div>
        {#if target}
          <div class="flex flex-wrap gap-2">
            {#if targetPlayable}
              <Button href={href.watch(target.id)} size="lg" class="h-auto min-h-9 max-w-full py-2 whitespace-normal">
                <PlayIcon data-icon="inline-start" />
                {resuming ? "Resume" : "Play"}
                {episodeName(target)}
              </Button>
            {:else if target.download && canDownload}
              <Button variant="outline" size="lg" href={href.activity()}>View download</Button>
            {:else if !target.download && canDownload}
              <FindAgainLink item={target} />
            {/if}
            <IdentifyButton item={target} />
            <FavoriteButton items={episodes} label="Favorite show" />
            {#if admin}
              <Button
                variant="outline"
                size="lg"
                disabled={transferring(episodes)}
                title={transferring(episodes) ? "Manage active transfers in Downloads before removing this selection" : undefined}
                onclick={() => confirmRemoval(episodes, show.title, true)}
              >
                <TrashIcon data-icon="inline-start" />Remove series
              </Button>
            {/if}
          </div>
        {/if}
      </header>

      {#if target}<MetadataDetails item={target} series />{/if}
    </DetailLayout>
  {/if}
  {#if admin && show}
    <RemovalDialog bind:open={removalOpen} itemIds={removal.ids} label={removal.label} onremoved={handleRemoved} />
  {/if}
</CatalogGate>
