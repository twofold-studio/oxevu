<script lang="ts">
  import CheckIcon from "@lucide/svelte/icons/check";
  import HardDriveOffIcon from "@lucide/svelte/icons/hard-drive";
  import Artwork from "$lib/components/Artwork.svelte";
  import { cn } from "$lib/utils";
  import ProgressLine from "./ProgressLine.svelte";

  // A poster cell in the hairline grid. Real sidecar art fills the frame; when
  // there is none (or it fails to load) the title itself is set as the cover
  // with the real file facts, so nothing on the card is invented.
  let {
    href,
    title,
    poster = null,
    facts = [],
    detail,
    progress = null,
    watched = false,
    fileRemoved = false,
  }: {
    href: string;
    title: string;
    poster?: string | null;
    facts?: string[];
    detail?: string | null;
    progress?: number | null;
    watched?: boolean;
    /** True when the item's local file has been removed. */
    fileRemoved?: boolean;
  } = $props();

  const size = $derived(
    title.length <= 14 ? "text-2xl" : title.length <= 32 ? "text-xl" : "text-lg",
  );
</script>

<li class="border-r border-b">
  <a
    {href}
    class="group flex h-full flex-col focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
  >
    <div class="relative">
      <!-- The caption below names the title, so the image itself is decorative. -->
      <Artwork
        src={poster}
        alt=""
        class="aspect-[2/3] border-b bg-card"
        imageClass="transition-[filter] duration-300 group-hover:brightness-110"
      >
        {#snippet fallback()}
          <div
            aria-hidden="true"
            class="absolute inset-0 flex flex-col justify-between bg-[radial-gradient(120%_80%_at_0%_100%,oklch(1_0_0/0.05),transparent_60%)] p-4"
          >
            <span class={cn("truncate font-mono text-[0.7rem] text-muted-foreground", watched && "pr-20")}>
              {facts.join("  ")}
            </span>
            <span
              class={cn(
                "line-clamp-5 leading-[1.12] font-semibold tracking-tight text-balance break-words text-foreground/85",
                size,
              )}
            >
              {title}
            </span>
          </div>
        {/snippet}
      </Artwork>
      {#if fileRemoved}
        <span
          class="absolute top-2 left-2 flex items-center gap-1 rounded-md bg-background/80 px-1.5 py-0.5 text-[0.7rem] text-muted-foreground backdrop-blur-sm"
        >
          <HardDriveOffIcon class="size-3" />
          No local file
        </span>
      {/if}
      {#if watched}
        <span
          class="absolute top-2 right-2 flex items-center gap-1 rounded-md bg-background/80 px-1.5 py-0.5 text-[0.7rem] text-foreground backdrop-blur-sm"
        >
          <CheckIcon class="size-3" />
          Watched
        </span>
      {/if}
      {#if progress !== null && !watched}
        <ProgressLine value={progress} class="absolute inset-x-0 bottom-0 h-1 bg-background/60" />
      {/if}
    </div>
    <div
      class="flex min-w-0 flex-1 flex-col gap-0.5 px-3 py-2.5 transition-colors duration-200 group-hover:bg-accent/40 sm:px-4"
    >
      <h3 class="truncate text-sm font-medium" {title}>{title}</h3>
      {#if detail}
        <p class="truncate text-xs text-muted-foreground">{detail}</p>
      {/if}
    </div>
  </a>
</li>
