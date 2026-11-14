<script lang="ts">
  import PlayIcon from "@lucide/svelte/icons/play";
  import FilmIcon from "@lucide/svelte/icons/film";
  import type { Item } from "$lib/api/types";
  import { href } from "$lib/state/router.svelte";
  import { episodeCaption } from "$lib/media/grouping";
  import { formatRuntime } from "$lib/media/format";
  import { resumePosition, watchedFraction } from "$lib/media/progress";
  import Artwork from "$lib/components/Artwork.svelte";
  import ProgressLine from "./ProgressLine.svelte";

  // A large 16:9 still for the episode a series would play next. Links to the
  // episode's details; the play affordance on the still is the same link.
  let { item }: { item: Item } = $props();

  const caption = $derived(episodeCaption(item));
  const resume = $derived(resumePosition(item));
  const fraction = $derived(watchedFraction(item));
  const remaining = $derived(
    resume !== null && item.duration_seconds
      ? `${formatRuntime(item.duration_seconds - resume) ?? "<1m"} left`
      : formatRuntime(item.duration_seconds),
  );
</script>

<a
  href={href.item(item.id)}
  class="group flex w-full max-w-[28rem] flex-col gap-2.5 rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-4 focus-visible:ring-offset-background"
>
  <div class="relative overflow-hidden rounded-xl border shadow-xl shadow-black/40">
    <!-- The caption names the episode, so the still is decorative. -->
    <Artwork
      src={item.thumbnail_url}
      alt=""
      class="aspect-video"
      imageClass="transition-[filter,transform] duration-300 group-hover:scale-[1.02] group-hover:brightness-110"
    >
      {#snippet fallback()}
        <span aria-hidden="true" class="absolute inset-0 flex items-center justify-center text-muted-foreground/50">
          <FilmIcon class="size-8" />
        </span>
      {/snippet}
    </Artwork>
    <span
      aria-hidden="true"
      class="absolute bottom-3 left-3 flex size-11 items-center justify-center rounded-full bg-background/80 text-foreground backdrop-blur-sm transition-colors group-hover:bg-foreground group-hover:text-background"
    >
      <PlayIcon class="size-4 fill-current" />
    </span>
    {#if fraction !== null && !item.completed}
      <ProgressLine value={fraction} class="absolute inset-x-0 bottom-0 h-1 bg-background/60" />
    {/if}
  </div>
  <!-- One understated line, "S1:E1 - Title"; truncation keeps the full text in `title`. -->
  <span class="flex min-w-0 items-baseline justify-between gap-3 text-sm">
    <span class="truncate text-foreground/85 group-hover:underline" title={caption}>{caption}</span>
    {#if remaining}<span class="shrink-0 text-xs text-muted-foreground tabular-nums">{remaining}</span>{/if}
  </span>
</a>
