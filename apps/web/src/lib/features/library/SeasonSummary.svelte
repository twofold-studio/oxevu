<script lang="ts">
  import type { Snippet } from "svelte";
  import CheckIcon from "@lucide/svelte/icons/check";
  import TvIcon from "@lucide/svelte/icons/tv";
  import Artwork from "$lib/components/Artwork.svelte";
  import { seasonLabel, type Season } from "$lib/media/grouping";
  import { plural } from "$lib/media/format";
  import { seasonRating } from "$lib/media/metadata";
  import MetaFact from "./metadata/MetaFact.svelte";
  import RatingBadge from "./metadata/RatingBadge.svelte";
  import ProgressLine from "./ProgressLine.svelte";

  // The heading block above a season's episodes: its own cover (only when it
  // differs from the series poster already shown in the hero), name, counts,
  // score, watched share, overview and season-scoped actions.
  let {
    season,
    seriesPoster,
    actions,
  }: {
    season: Season;
    seriesPoster: string | null;
    actions?: Snippet<[Season]>;
  } = $props();

  const label = $derived(seasonLabel(season.number));
  const watched = $derived(season.episodes.filter((e) => e.completed).length);
  const rating = $derived(seasonRating(season.episodes[0]));
  const overview = $derived(season.episodes[0]?.season_overview);
  const ownCover = $derived(season.posterUrl !== null && season.posterUrl !== seriesPoster);
</script>

<div class="flex gap-4">
  {#if ownCover}
    <!-- The heading beside it names the season, so the cover is decorative. -->
    <Artwork src={season.posterUrl} alt="" class="hidden aspect-[2/3] w-20 shrink-0 self-start rounded-md border bg-card sm:block" />
  {/if}
  <div class="flex min-w-0 flex-1 flex-col gap-3">
    <div class="flex flex-wrap items-start justify-between gap-x-6 gap-y-3">
      <div class="flex min-w-0 flex-col gap-1">
        <h3 class="text-lg font-semibold tracking-tight">{label}</h3>
        <ul aria-label={`${label} details`} class="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-muted-foreground">
          <MetaFact icon={TvIcon}>{plural(season.episodes.length, "episode")}</MetaFact>
          {#if watched > 0}
            <MetaFact icon={CheckIcon}>{watched === season.episodes.length ? "All watched" : `${watched} of ${season.episodes.length} watched`}</MetaFact>
          {/if}
          {#if rating}<li><RatingBadge {rating} /></li>{/if}
        </ul>
      </div>
      {#if actions}<div class="flex flex-wrap items-center gap-2">{@render actions(season)}</div>{/if}
    </div>
    {#if watched > 0 && watched < season.episodes.length}
      <ProgressLine value={watched / season.episodes.length} class="h-1 max-w-xs rounded-full" />
    {/if}
    {#if overview}<p class="max-w-3xl text-sm leading-6 text-muted-foreground">{overview}</p>{/if}
  </div>
</div>
