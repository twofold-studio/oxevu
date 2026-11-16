<script lang="ts">
  import type { Snippet } from "svelte";
  import CheckIcon from "@lucide/svelte/icons/check";
  import * as Tabs from "$lib/components/ui/tabs";
  import SwapView from "$lib/components/SwapView.svelte";
  import { seasonKey, seasonLabel, type Season } from "$lib/media/grouping";
  import { plural } from "$lib/media/format";
  import EpisodeList from "./EpisodeList.svelte";
  import SeasonSummary from "./SeasonSummary.svelte";

  // A series' episodes, one season at a time. Seasons are a compact underline
  // tab strip directly above the selected season's summary and episodes, so
  // switching seasons never leaves the episode area. bits-ui supplies the
  // tablist semantics, roving focus and arrow keys; the strip scrolls
  // sideways when a long-running series has more seasons than fit. A
  // single-season series has nothing to choose and shows no strip.
  // SwapView crossfades the old season out and the new one in without the
  // section collapsing in between.
  let {
    seasons,
    value,
    onValueChange,
    upNextId,
    seriesPoster,
    actions,
  }: {
    seasons: Season[];
    value: string;
    onValueChange: (value: string) => void;
    /** The episode the show's Play button targets, marked in its list. */
    upNextId?: string;
    seriesPoster: string | null;
    /** Season-scoped actions (favorite, remove) beside the season heading. */
    actions?: Snippet<[Season]>;
  } = $props();

  let list = $state<HTMLElement | null>(null);

  // Keep the selected tab visible in a scrolled strip (deep links, Back, and
  // keyboard selection past the edge). Only the strip scrolls, never the page.
  $effect(() => {
    if (!list || !value) return;
    const active = list.querySelector<HTMLElement>("[data-state='active']");
    if (!active) return;
    const left = active.offsetLeft - (list.clientWidth - active.offsetWidth) / 2;
    list.scrollTo({ left: Math.max(0, left) });
  });

  const watchedIn = (season: Season) => season.episodes.filter((e) => e.completed).length;
</script>

{#snippet panel(key: string)}
  {@const season = seasons.find((s) => seasonKey(s) === key)}
  {#if season}
    <div class="flex flex-col gap-5">
      <SeasonSummary {season} {seriesPoster} {actions} />
      <EpisodeList episodes={season.episodes} label={seasonLabel(season.number)} {upNextId} />
    </div>
  {/if}
{/snippet}

{#if seasons.length > 1}
  <Tabs.Root {value} {onValueChange} class="gap-6">
    <Tabs.List
      bind:ref={list}
      variant="line"
      aria-label="Seasons"
      class="relative h-auto w-full justify-start gap-0 overflow-x-auto rounded-none p-0 [scrollbar-width:thin] shadow-[inset_0_-1px_0_0_var(--color-border)]"
    >
      {#each seasons as season (seasonKey(season))}
        {@const watched = watchedIn(season)}
        {@const done = watched === season.episodes.length}
        {@const hasUpNext = upNextId !== undefined && season.episodes.some((e) => e.id === upNextId)}
        <Tabs.Trigger
          value={seasonKey(season)}
          class="h-10 flex-none px-3 group-data-horizontal/tabs:after:bottom-0 sm:px-4"
        >
          {#if hasUpNext}
            <span aria-hidden="true" class="size-1.5 shrink-0 rounded-full bg-brand" title="Up next"></span>
          {/if}
          {seasonLabel(season.number)}
          {#if done}
            <CheckIcon aria-hidden="true" class="size-3.5 text-muted-foreground" />
          {:else}
            <span aria-hidden="true" class="text-xs font-normal text-muted-foreground tabular-nums">{season.episodes.length}</span>
          {/if}
          <span class="sr-only">
            , {plural(season.episodes.length, "episode")}{watched > 0
              ? done
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
        {#snippet children(key)}{@render panel(key)}{/snippet}
      </SwapView>
    </Tabs.Content>
  </Tabs.Root>
{:else if seasons.length === 1}
  {@render panel(seasonKey(seasons[0]))}
{/if}
