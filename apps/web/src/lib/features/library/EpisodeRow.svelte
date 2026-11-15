<script lang="ts">
  import PlayIcon from "@lucide/svelte/icons/play";
  import CheckIcon from "@lucide/svelte/icons/check";
  import FilmIcon from "@lucide/svelte/icons/film";
  import { Button } from "$lib/components/ui/button";
  import type { Item } from "$lib/api/types";
  import { href } from "$lib/state/router.svelte";
  import { episodeName, episodeNumberLabel } from "$lib/media/grouping";
  import { formatClock, formatRuntime } from "$lib/media/format";
  import { resumePosition, watchedFraction } from "$lib/media/progress";
  import Artwork from "$lib/components/Artwork.svelte";
  import ProgressLine from "./ProgressLine.svelte";
  import ItemActions from "./ItemActions.svelte";
  import FlagIcon from "@lucide/svelte/icons/flag";
  import { endsAt, itemRating } from "$lib/media/metadata";
  import { now } from "$lib/state/clock.svelte";
  import MetaFact from "./metadata/MetaFact.svelte";
  import RatingBadge from "./metadata/RatingBadge.svelte";

  let {
    item,
    showContext = false,
    upNext = false,
  }: {
    item: Item;
    showContext?: boolean;
    /** This is the episode the series' Play button would start or resume. */
    upNext?: boolean;
  } = $props();

  // Inside a season list the season is already on screen; mixed lists
  // (Continue watching, search) name it.
  const number = $derived(episodeNumberLabel(item, { withSeason: showContext }));
  const name = $derived(episodeName(item));
  // Episodes use their own still; movies (in Continue watching) fall back to the backdrop.
  const still = $derived(item.thumbnail_url ?? (item.kind === "movie" ? item.backdrop_url : null));
  const resume = $derived(resumePosition(item));
  const fraction = $derived(watchedFraction(item));
  const rating = $derived(itemRating(item));
  const finish = $derived(endsAt(item.duration_seconds, resume, now()));
  const status = $derived(
    item.completed
      ? "Watched"
      : resume !== null && item.duration_seconds
        ? `${formatRuntime(item.duration_seconds - resume) ?? "<1m"} left`
        : resume !== null
          ? `Stopped at ${formatClock(resume)}`
          : null,
  );
</script>

<!-- Up next carries a bright edge marker plus a text label, so it reads with
     motion and colour turned off and is spoken as part of the link. -->
<li
  class={[
    "relative flex flex-wrap items-center gap-3 px-3 py-3 transition-colors duration-(--motion-control) ease-settle hover:bg-accent/40 has-[a:active]:bg-accent/60 sm:flex-nowrap sm:gap-4 sm:px-4",
    upNext && "bg-accent/25 before:absolute before:inset-y-0 before:left-0 before:w-0.5 before:bg-brand",
  ]}
>
  <!-- The title beside it names the episode, so the still is decorative. -->
  <Artwork src={still} alt="" class="aspect-video w-28 shrink-0 rounded-md border sm:w-44 lg:w-52">
    {#snippet fallback()}
      <span aria-hidden="true" class="absolute inset-0 flex items-center justify-center text-muted-foreground/50">
        <FilmIcon class="size-4" />
      </span>
    {/snippet}
  </Artwork>
  <a
    href={href.item(item.id)}
    class="flex min-w-0 flex-1 flex-col gap-0.5 max-sm:basis-[calc(100%-7.75rem)] after:absolute after:inset-0 focus-visible:outline-none focus-visible:after:ring-2 focus-visible:after:ring-ring focus-visible:after:ring-inset"
  >
    {#if (showContext && item.show_title) || number || upNext}
      <span class="flex min-w-0 gap-2 text-xs text-muted-foreground">
        {#if number}<span class="shrink-0 tabular-nums">{number}</span>{/if}
        {#if upNext}<span class="shrink-0 font-medium text-brand">{resume !== null ? "Continue" : "Up next"}</span>{/if}
        {#if showContext && item.show_title}<span class="truncate">{item.show_title}</span>{/if}
      </span>
    {/if}
    <span class="line-clamp-2 font-medium break-words sm:line-clamp-1">{name}</span>
    {#if rating || finish}
      <ul aria-label="Details" class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
        {#if rating}<li><RatingBadge {rating} /></li>{/if}
        {#if finish}<MetaFact icon={FlagIcon} label={`Ends at ${finish}`}>{finish}</MetaFact>{/if}
      </ul>
    {/if}
    {#if item.overview}<span class="line-clamp-2 text-sm text-muted-foreground">{item.overview}</span>{/if}
    <!-- Phones hide the status column; keep watched/remaining visible here. -->
    {#if status}
      <span class="flex items-center gap-1 text-xs text-muted-foreground sm:hidden">
        {#if item.completed}<CheckIcon class="size-3" />{/if}
        {status}
      </span>
    {/if}
  </a>
  <span class="hidden shrink-0 items-center gap-1.5 text-sm text-muted-foreground sm:flex">
    {#if item.completed}<CheckIcon class="size-3.5" />{/if}
    {status ?? formatRuntime(item.duration_seconds) ?? ""}
  </span>
  <!-- Sits above the row link overlay so it stays independently clickable.
       Phones wrap the actions onto their own right-aligned line so the title
       and facts keep the width beside the still. -->
  <div class="ms-auto flex shrink-0 items-center sm:ms-0">
    <ItemActions {item} />
    <Button
      href={href.watch(item.id)}
      variant="ghost"
      size="icon"
      class="relative z-10 shrink-0"
      aria-label={`${resume !== null ? "Resume" : "Play"} ${name}`}
    >
      <PlayIcon />
    </Button>
  </div>
  {#if fraction !== null && !item.completed}
    <ProgressLine value={fraction} class="absolute inset-x-0 bottom-0" />
  {/if}
</li>
