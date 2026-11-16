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
  import { fileAvailable } from "$lib/state/catalog.svelte";

  // Details and playback have separate links so keyboard users can choose either.
  let { item }: { item: Item } = $props();

  const caption = $derived(episodeCaption(item));
  const resume = $derived(resumePosition(item));
  const fraction = $derived(watchedFraction(item));
  const remaining = $derived(
    resume !== null && item.duration_seconds
      ? `${formatRuntime(item.duration_seconds - resume) ?? "<1m"} left`
      : formatRuntime(item.duration_seconds),
  );
  const hasFile = $derived(fileAvailable(item));
</script>

<div
  class="group flex w-full max-w-[28rem] flex-col gap-2.5 rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-4 focus-visible:ring-offset-background"
>
  <div class="relative overflow-hidden rounded-xl border shadow-xl shadow-black/40">
    <a href={href.item(item.id)} aria-label={`Episode information: ${caption}`} class="block focus-visible:ring-2 focus-visible:ring-ring">
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
    </a>
    {#if hasFile}
      <a
        href={href.watch(item.id)}
        aria-label={`${resume !== null ? "Resume" : "Play"} ${caption}`}
        class="press absolute right-3 bottom-3 flex size-12 items-center justify-center rounded-full bg-foreground text-background shadow-lg transition-transform duration-180 hover:scale-110 focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-ring motion-reduce:transform-none"
      >
        <PlayIcon class="size-4 fill-current" />
      </a>
    {/if}
    {#if fraction !== null && !item.completed}
      <ProgressLine value={fraction} class="absolute inset-x-0 bottom-0 h-1 bg-background/60" />
    {/if}
  </div>
  <!-- One understated line, "Season 2, Episode 5 - Title" (the card can point
       outside the open season tab, so it names the season); truncation keeps the full text in `title`. -->
  <span class="flex min-w-0 items-baseline justify-between gap-3 text-sm">
    <a href={href.item(item.id)} class="line-clamp-2 min-w-0 text-foreground/85 hover:underline" title={caption}>{caption}</a>
    {#if remaining}<span class="shrink-0 text-xs text-muted-foreground tabular-nums">{remaining}</span>{/if}
  </span>
</div>
