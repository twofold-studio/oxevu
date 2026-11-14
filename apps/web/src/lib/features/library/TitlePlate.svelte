<script lang="ts">
  import CheckIcon from "@lucide/svelte/icons/check";
  import { cn } from "$lib/utils";
  import ProgressLine from "./ProgressLine.svelte";

  // Phase one has no artwork, so the title itself is the cover: set large,
  // sized by length, with real file facts above it. Nothing here is invented.
  let {
    href,
    title,
    facts = [],
    detail,
    progress = null,
    watched = false,
  }: {
    href: string;
    title: string;
    facts?: string[];
    detail?: string | null;
    progress?: number | null;
    watched?: boolean;
  } = $props();

  const size = $derived(
    title.length <= 14 ? "text-2xl" : title.length <= 32 ? "text-xl" : "text-lg",
  );
</script>

<li class="border-r border-b">
  <a
    {href}
    class="group relative flex aspect-[4/3] flex-col justify-between gap-3 p-4 transition-colors duration-200 hover:bg-accent/40 focus-visible:bg-accent/40 focus-visible:outline-none focus-visible:ring-inset focus-visible:ring-2 focus-visible:ring-ring sm:p-5"
  >
    <div class="flex min-h-5 items-start justify-between gap-2">
      <span class="truncate font-mono text-[0.7rem] text-muted-foreground">{facts.join("  ")}</span>
      {#if watched}
        <span class="flex shrink-0 items-center gap-1 text-[0.7rem] text-muted-foreground">
          <CheckIcon class="size-3.5" />
          Watched
        </span>
      {/if}
    </div>
    <div class="flex min-w-0 flex-col gap-1.5">
      <h3 class={cn("line-clamp-3 leading-[1.12] font-semibold tracking-tight text-balance break-words", size)}>
        {title}
      </h3>
      {#if detail}
        <p class="truncate text-sm text-muted-foreground">{detail}</p>
      {/if}
    </div>
    {#if progress !== null && !watched}
      <ProgressLine value={progress} class="absolute inset-x-0 bottom-0" />
    {/if}
  </a>
</li>
