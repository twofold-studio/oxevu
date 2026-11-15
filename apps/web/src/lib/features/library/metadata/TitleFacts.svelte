<script lang="ts">
  import type { Snippet } from "svelte";
  import CalendarIcon from "@lucide/svelte/icons/calendar";
  import ClockIcon from "@lucide/svelte/icons/clock";
  import FlagIcon from "@lucide/svelte/icons/flag";
  import type { Item } from "$lib/api/types";
  import { formatRuntime } from "$lib/media/format";
  import { endsAt, itemRating, present, yearSpan, type Rating } from "$lib/media/metadata";
  import { resumePosition } from "$lib/media/progress";
  import { now } from "$lib/state/clock.svelte";
  import MetaFact from "./MetaFact.svelte";
  import RatingBadge from "./RatingBadge.svelte";

  // The compact facts line under a title: years, certificate, score, length
  // and finish time, followed by page-specific facts passed as <li> children.
  // `rating` overrides the record's own score (a season page shows the
  // season's score); pass null to hide it.
  let {
    item,
    series = false,
    rating: override,
    children,
  }: { item: Item; series?: boolean; rating?: Rating | null; children?: Snippet } = $props();

  const years = $derived(yearSpan(item, series));
  const certificate = $derived(present(series ? item.show_content_rating : item.content_rating));
  const rating = $derived(override === undefined ? itemRating(item, series) : override);
  const runtime = $derived(series ? null : formatRuntime(item.duration_seconds));
  const finish = $derived(series ? null : endsAt(item.duration_seconds, resumePosition(item), now()));
</script>

<ul aria-label="Details" class="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-muted-foreground">
  {#if years}<MetaFact icon={CalendarIcon} label={years.label}>{years.text}</MetaFact>{/if}
  {#if certificate}
    <MetaFact label={`Rated ${certificate}`}>
      <span class="rounded-[3px] border border-foreground/30 px-1 py-px text-xs leading-none font-medium text-foreground/85">{certificate}</span>
    </MetaFact>
  {/if}
  {#if rating}<li><RatingBadge {rating} /></li>{/if}
  {#if runtime}<MetaFact icon={ClockIcon} label={`Runtime ${runtime}`}>{runtime}</MetaFact>{/if}
  {#if finish}<MetaFact icon={FlagIcon} label={`Ends at ${finish}`}>{finish}</MetaFact>{/if}
  {@render children?.()}
</ul>
