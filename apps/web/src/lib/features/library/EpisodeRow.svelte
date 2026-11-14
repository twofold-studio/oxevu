<script lang="ts">
  import PlayIcon from "@lucide/svelte/icons/play";
  import CheckIcon from "@lucide/svelte/icons/check";
  import FilmIcon from "@lucide/svelte/icons/film";
  import { Button } from "$lib/components/ui/button";
  import type { Item } from "$lib/api/types";
  import { href } from "$lib/state/router.svelte";
  import { episodeCode } from "$lib/media/grouping";
  import { formatClock, formatRuntime } from "$lib/media/format";
  import { resumePosition, watchedFraction } from "$lib/media/progress";
  import Artwork from "$lib/components/Artwork.svelte";
  import ProgressLine from "./ProgressLine.svelte";

  let { item, showContext = false }: { item: Item; showContext?: boolean } = $props();

  const code = $derived(episodeCode(item));
  // Episodes use their own still; movies (in Continue watching) fall back to the backdrop.
  const still = $derived(item.thumbnail_url ?? (item.kind === "movie" ? item.backdrop_url : null));
  const resume = $derived(resumePosition(item));
  const fraction = $derived(watchedFraction(item));
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

<li class="relative flex items-center gap-3 px-3 py-2.5 transition-colors duration-200 hover:bg-accent/40 sm:gap-4 sm:px-4">
  <!-- The title beside it names the episode, so the still is decorative. -->
  <Artwork src={still} alt="" class="aspect-video w-24 shrink-0 rounded-md border sm:w-36">
    {#snippet fallback()}
      <span aria-hidden="true" class="absolute inset-0 flex items-center justify-center text-muted-foreground/50">
        <FilmIcon class="size-4" />
      </span>
    {/snippet}
  </Artwork>
  <a
    href={href.item(item.id)}
    class="flex min-w-0 flex-1 flex-col gap-0.5 after:absolute after:inset-0 focus-visible:outline-none focus-visible:after:ring-2 focus-visible:after:ring-ring focus-visible:after:ring-inset"
  >
    {#if (showContext && item.show_title) || code}
      <span class="flex min-w-0 gap-2 text-xs text-muted-foreground">
        {#if code}<span class="shrink-0 font-mono tabular-nums">{code}</span>{/if}
        {#if showContext && item.show_title}<span class="truncate">{item.show_title}</span>{/if}
      </span>
    {/if}
    <span class="line-clamp-2 font-medium break-words sm:line-clamp-1">{item.title}</span>
  </a>
  <span class="hidden shrink-0 items-center gap-1.5 text-sm text-muted-foreground sm:flex">
    {#if item.completed}<CheckIcon class="size-3.5" />{/if}
    {status ?? formatRuntime(item.duration_seconds) ?? ""}
  </span>
  <!-- Sits above the row link overlay so it stays independently clickable. -->
  <Button
    href={href.watch(item.id)}
    variant="ghost"
    size="icon"
    class="relative z-10 shrink-0"
    aria-label={`${resume !== null ? "Resume" : "Play"} ${item.title}`}
  >
    <PlayIcon />
  </Button>
  {#if fraction !== null && !item.completed}
    <ProgressLine value={fraction} class="absolute inset-x-0 bottom-0" />
  {/if}
</li>
