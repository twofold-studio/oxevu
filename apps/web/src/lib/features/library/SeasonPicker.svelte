<script lang="ts">
  import { Tabs } from "bits-ui";
  import Artwork from "$lib/components/Artwork.svelte";
  import SwapView from "$lib/components/SwapView.svelte";
  import { seasonKey, seasonLabel, type Season } from "$lib/media/grouping";
  import { plural } from "$lib/media/format";
  import EpisodeList from "./EpisodeList.svelte";
  import ProgressLine from "./ProgressLine.svelte";

  // Season covers as tabs: bits-ui supplies the tablist semantics, roving focus
  // and arrow-key selection; each tab is a 2:3 cover with its episode count.
  // Covers come from grouping (season folder art, else the show poster); with
  // neither, the season name is set on a plain card instead of invented art.
  // One panel holds the selected season; SwapView fades the old episode list
  // out and the new one in without the section collapsing in between.
  let {
    seasons,
    value,
    onValueChange,
    upNextId,
  }: {
    seasons: Season[];
    value: string;
    onValueChange: (value: string) => void;
    /** The episode the show's Play button targets, marked in its list. */
    upNextId?: string;
  } = $props();

  const watchedIn = (season: Season) => season.episodes.filter((e) => e.completed).length;
</script>

<Tabs.Root {value} {onValueChange} class="flex flex-col gap-6">
  <!-- Phones fill two flexible columns: a fixed track max makes auto-fill
       count columns at that max, which left one huge cover per row there. -->
  <Tabs.List
    aria-label="Seasons"
    class="grid grid-cols-[repeat(auto-fill,minmax(min(7.5rem,100%),1fr))] gap-x-4 gap-y-5 sm:grid-cols-[repeat(auto-fill,minmax(min(8rem,100%),12.5rem))] sm:gap-x-5"
  >
    {#each seasons as season (seasonKey(season))}
      {@const label = seasonLabel(season.number)}
      {@const watched = watchedIn(season)}
      {@const hasUpNext = upNextId !== undefined && season.episodes.some((e) => e.id === upNextId)}
      <Tabs.Trigger
        value={seasonKey(season)}
        class="group/season flex min-w-0 flex-col items-center gap-2.5 rounded-xl text-center outline-none focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-ring"
      >
        <!-- Selection reads without motion: ring, full brightness and a bright
             label. `lift` only adds the 2px hover raise and press. -->
        <span
          class="lift relative block w-full rounded-lg ring-offset-2 ring-offset-background group-data-[state=active]/season:ring-2 group-data-[state=active]/season:ring-foreground"
        >
          <!-- The label below names the season, so the cover is decorative. -->
          <Artwork
            src={season.posterUrl}
            alt=""
            class="aspect-[2/3] rounded-lg border bg-card shadow-lg shadow-black/40"
            imageClass="brightness-[0.75] transition-[filter] duration-(--motion-fade) ease-settle group-hover/season:brightness-100 group-data-[state=active]/season:brightness-100"
          >
            {#snippet fallback()}
              <span
                aria-hidden="true"
                class="absolute inset-0 flex items-end bg-[radial-gradient(120%_80%_at_0%_100%,oklch(1_0_0/0.05),transparent_60%)] p-4 text-lg leading-tight font-semibold tracking-tight text-balance text-foreground/80"
              >
                {label}
              </span>
            {/snippet}
          </Artwork>
          <span
            aria-hidden="true"
            class="absolute top-2 right-2 flex h-6 min-w-6 items-center justify-center rounded-full bg-primary px-1.5 text-xs font-medium text-primary-foreground tabular-nums shadow-md shadow-black/40"
          >
            {season.episodes.length}
          </span>
          {#if watched > 0}
            <!-- Real watched share for the season; spoken in the sr-only text below. -->
            <span aria-hidden="true" class="absolute inset-x-0 bottom-0 overflow-hidden rounded-b-lg">
              <ProgressLine value={watched / season.episodes.length} class="h-1 bg-background/60" />
            </span>
          {/if}
        </span>
        <span
          class="flex max-w-full items-center gap-1.5 text-sm text-muted-foreground transition-colors group-hover/season:text-foreground group-data-[state=active]/season:font-medium group-data-[state=active]/season:text-foreground"
        >
          {#if hasUpNext}
            <span aria-hidden="true" class="size-1.5 shrink-0 rounded-full bg-brand" title="Up next"></span>
          {/if}
          <span class="truncate">{label}</span>
        </span>
        <span class="sr-only">
          , {plural(season.episodes.length, "episode")}{watched > 0
            ? watched === season.episodes.length
              ? ", all watched"
              : `, ${watched} watched`
            : ""}{hasUpNext ? ", has up next" : ""}
        </span>
      </Tabs.Trigger>
    {/each}
  </Tabs.List>
  <Tabs.Content
    {value}
    class="rounded-xl outline-none focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-ring"
  >
    <SwapView key={value}>
      {#snippet children(key)}
        {@const season = seasons.find((s) => seasonKey(s) === key)}
        {#if season}
          <EpisodeList episodes={season.episodes} label={seasonLabel(season.number)} {upNextId} />
        {/if}
      {/snippet}
    </SwapView>
  </Tabs.Content>
</Tabs.Root>
