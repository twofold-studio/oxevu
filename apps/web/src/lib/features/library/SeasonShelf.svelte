<script lang="ts">
  import Artwork from "$lib/components/Artwork.svelte";
  import { href } from "$lib/state/router.svelte";
  import { seasonKey, seasonLabel, type Season } from "$lib/media/grouping";
  import { plural } from "$lib/media/format";
  import ProgressLine from "./ProgressLine.svelte";

  // Season covers on a series overview, each linking to its season page.
  // `class` sets the track width so the caller can size covers to the space
  // it has; the label below names the season, so the cover is decorative.
  let { slug, seasons, class: className = "" }: { slug: string; seasons: Season[]; class?: string } = $props();

  const watchedIn = (season: Season) => season.episodes.filter((e) => e.completed).length;
</script>

<ul class={`grid gap-x-4 gap-y-5 ${className}`}>
  {#each seasons as season (seasonKey(season))}
    {@const label = seasonLabel(season.number)}
    {@const watched = watchedIn(season)}
    <li class="min-w-0">
      <a href={href.show(slug, seasonKey(season))} class="group/season lift press flex flex-col gap-2 rounded-lg outline-none focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-ring">
        <span class="relative block">
          <Artwork
            src={season.posterUrl}
            alt=""
            class="aspect-[2/3] rounded-lg border bg-card shadow-lg shadow-black/40"
            imageClass="transition-[filter] duration-(--motion-fade) ease-settle group-hover/season:brightness-110"
          >
            {#snippet fallback()}
              <span aria-hidden="true" class="absolute inset-0 flex items-end p-3 text-base leading-tight font-semibold tracking-tight text-balance text-foreground/80">{label}</span>
            {/snippet}
          </Artwork>
          {#if watched > 0}
            <span aria-hidden="true" class="absolute inset-x-0 bottom-0 overflow-hidden rounded-b-lg">
              <ProgressLine value={watched / season.episodes.length} class="h-1 bg-background/60" />
            </span>
          {/if}
        </span>
        <span class="flex min-w-0 flex-col">
          <span class="truncate text-sm font-medium">{label}</span>
          <span class="truncate text-xs text-muted-foreground">
            {plural(season.episodes.length, "episode")}{watched > 0 ? (watched === season.episodes.length ? ", all watched" : `, ${watched} watched`) : ""}
          </span>
        </span>
      </a>
    </li>
  {/each}
</ul>
