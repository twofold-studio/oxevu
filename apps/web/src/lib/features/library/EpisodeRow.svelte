<script lang="ts">
  import PlayIcon from "@lucide/svelte/icons/play";
  import CheckIcon from "@lucide/svelte/icons/check";
  import { Button } from "$lib/components/ui/button";
  import type { Item } from "$lib/api/types";
  import { href } from "$lib/state/router.svelte";
  import { episodeCode } from "$lib/media/grouping";
  import { formatClock, formatRuntime } from "$lib/media/format";
  import { resumePosition, watchedFraction } from "$lib/media/progress";
  import ProgressLine from "./ProgressLine.svelte";

  let { item, showContext = false }: { item: Item; showContext?: boolean } = $props();

  const code = $derived(episodeCode(item));
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

<li class="relative flex items-center gap-4 px-4 py-3 transition-colors duration-200 hover:bg-accent/40 sm:px-5">
  <span class="w-12 shrink-0 font-mono sm:w-14 text-xs text-muted-foreground tabular-nums">{code ?? (item.kind === "movie" ? "Movie" : "—")}</span>
  <a
    href={href.item(item.id)}
    class="flex min-w-0 flex-1 flex-col gap-0.5 after:absolute after:inset-0 focus-visible:outline-none focus-visible:after:ring-2 focus-visible:after:ring-ring focus-visible:after:ring-inset"
  >
    {#if showContext && item.show_title}
      <span class="truncate text-xs text-muted-foreground">{item.show_title}</span>
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
