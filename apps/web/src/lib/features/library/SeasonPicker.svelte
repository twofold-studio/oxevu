<script lang="ts">
  import type { Snippet } from "svelte";
  import Artwork from "$lib/components/Artwork.svelte";
  import SwapView from "$lib/components/SwapView.svelte";
  import { seasonKey, seasonLabel, type Season } from "$lib/media/grouping";
  import { plural } from "$lib/media/format";
  import EpisodeList from "./EpisodeList.svelte";
  import SeasonSummary from "./SeasonSummary.svelte";

  let { seasons, value, onValueChange, upNextId, seriesPoster, actions }: {
    seasons: Season[];
    value: string;
    onValueChange: (value: string) => void;
    upNextId?: string;
    seriesPoster: string | null;
    actions?: Snippet<[Season]>;
  } = $props();
</script>

<div class="flex flex-col gap-6">
  <div class="grid grid-cols-[repeat(auto-fill,minmax(8rem,1fr))] gap-4 sm:grid-cols-[repeat(auto-fill,minmax(10rem,1fr))]" aria-label="Seasons">
    {#each seasons as season (seasonKey(season))}
      {@const selected = seasonKey(season) === value}
      {@const watched = season.episodes.filter((e) => e.completed).length}
      {@const hasUpNext = season.episodes.some((e) => e.id === upNextId)}
      <button type="button" aria-pressed={selected} onclick={() => onValueChange(seasonKey(season))}
        class="max-w-52 overflow-hidden rounded-xl border bg-card text-left transition-colors hover:border-foreground/50 focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-ring motion-reduce:transition-none {selected ? 'border-brand ring-1 ring-brand' : 'border-border'}">
        <Artwork src={season.posterUrl ?? seriesPoster} alt="" class="aspect-[2/3] w-full">
          {#snippet fallback()}<div class="flex h-full items-center justify-center bg-muted text-muted-foreground">{seasonLabel(season.number)}</div>{/snippet}
        </Artwork>
        <div class="space-y-1 p-3">
          <div class="flex items-center gap-2 font-medium">
            {#if hasUpNext}<span class="size-1.5 rounded-full bg-brand" aria-label="Up next"></span>{/if}
            {seasonLabel(season.number)}
          </div>
          <p class="text-xs text-muted-foreground">{plural(season.episodes.length, "episode")}</p>
          {#if watched > 0}<p class="text-xs text-muted-foreground">{watched === season.episodes.length ? "All watched" : `${watched} watched`}</p>{/if}
        </div>
      </button>
    {/each}
  </div>
  <SwapView key={value}>
    {#snippet children(key)}
      {@const season = seasons.find((s) => seasonKey(s) === key)}
      {#if season}
        <div class="flex flex-col gap-5">
          <SeasonSummary {season} {seriesPoster} {actions} />
          <EpisodeList episodes={season.episodes} label={seasonLabel(season.number)} {upNextId} />
        </div>
      {/if}
    {/snippet}
  </SwapView>
</div>
