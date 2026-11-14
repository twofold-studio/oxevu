<script lang="ts">
  import { Tabs } from "bits-ui";
  import Artwork from "$lib/components/Artwork.svelte";
  import { seasonKey, seasonLabel, type Season } from "$lib/media/grouping";
  import { plural } from "$lib/media/format";
  import EpisodeList from "./EpisodeList.svelte";

  // Season covers as tabs: bits-ui supplies the tablist semantics, roving focus
  // and arrow-key selection; each tab is a 2:3 cover with its episode count.
  // Covers come from grouping (season folder art, else the show poster); with
  // neither, the season name is set on a plain card instead of invented art.
  let {
    seasons,
    value,
    onValueChange,
  }: {
    seasons: Season[];
    value: string;
    onValueChange: (value: string) => void;
  } = $props();
</script>

<Tabs.Root {value} {onValueChange} class="flex flex-col gap-6">
  <Tabs.List
    aria-label="Seasons"
    class="grid grid-cols-[repeat(auto-fill,minmax(min(8rem,100%),12.5rem))] gap-x-4 gap-y-5 sm:gap-x-5"
  >
    {#each seasons as season (seasonKey(season))}
      {@const label = seasonLabel(season.number)}
      <Tabs.Trigger
        value={seasonKey(season)}
        class="group/season flex min-w-0 flex-col items-center gap-2.5 rounded-xl text-center outline-none focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-ring"
      >
        <span
          class="relative block w-full rounded-lg ring-offset-2 ring-offset-background transition-shadow group-data-[state=active]/season:ring-2 group-data-[state=active]/season:ring-foreground"
        >
          <!-- The label below names the season, so the cover is decorative. -->
          <Artwork
            src={season.posterUrl}
            alt=""
            class="aspect-[2/3] rounded-lg border bg-card shadow-lg shadow-black/40"
            imageClass="brightness-[0.8] transition-[filter] duration-300 group-hover/season:brightness-100 group-data-[state=active]/season:brightness-100"
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
        </span>
        <span
          class="max-w-full truncate text-sm text-muted-foreground transition-colors group-hover/season:text-foreground group-data-[state=active]/season:font-medium group-data-[state=active]/season:text-foreground"
        >
          {label}
        </span>
        <span class="sr-only">, {plural(season.episodes.length, "episode")}</span>
      </Tabs.Trigger>
    {/each}
  </Tabs.List>
  {#each seasons as season (seasonKey(season))}
    <Tabs.Content value={seasonKey(season)} class="outline-none">
      <EpisodeList episodes={season.episodes} label={seasonLabel(season.number)} />
    </Tabs.Content>
  {/each}
</Tabs.Root>
