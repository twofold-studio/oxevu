<script lang="ts">
  import type { Snippet } from "svelte";
  import CheckIcon from "@lucide/svelte/icons/check";
  import HardDriveOffIcon from "@lucide/svelte/icons/hard-drive";
  import Artwork from "$lib/components/Artwork.svelte";
  import DownloadBadge from "./DownloadBadge.svelte";
  import type { DownloadStatus } from "$lib/api/types";
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
    inProgress = progress !== null && !watched,
    fileRemoved = false,
    download = null,
    needsIdentification = false,
    badges,
  }: {
    href: string;
    title: string;
    poster?: string | null;
    facts?: string[];
    detail?: string | null;
    progress?: number | null;
    watched?: boolean;
    inProgress?: boolean;
    /** True when the item's local file has been removed. */
    fileRemoved?: boolean;
    download?: DownloadStatus | null;
    needsIdentification?: boolean;
    badges?: Snippet;
  } = $props();

  const size = $derived(
    title.length <= 14 ? "text-2xl" : title.length <= 32 ? "text-xl" : "text-lg",
  );
</script>

<li class="min-w-0">
  <a
    {href}
    class="group flex h-full flex-col gap-2 rounded-lg outline-offset-4 focus-visible:outline-2 focus-visible:outline-ring"
  >
    <div class="relative overflow-hidden rounded-lg border">
      <!-- The caption below names the title, so the image itself is decorative. -->
      <Artwork
        src={poster}
        alt=""
        class="aspect-[2/3] bg-card transition-opacity group-hover:opacity-85"
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
      {#if fileRemoved && !download}
        <span
          class="absolute top-2 left-2 flex items-center gap-1 rounded-md bg-background/80 px-1.5 py-0.5 text-[0.7rem] text-muted-foreground backdrop-blur-sm"
        >
          <HardDriveOffIcon class="size-3" />
          No local file
        </span>
      {:else if download}
        <span
          class="absolute top-2 left-2"
        >
          <DownloadBadge {download} compact />
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
      class="flex min-w-0 flex-1 flex-col gap-1"
    >
      <h3 class="line-clamp-2 font-medium leading-snug" {title}>{title}</h3>
      {#if inProgress && !watched}<p class="text-xs font-medium text-muted-foreground">In progress</p>{/if}
      {#if needsIdentification}<p class="text-xs font-medium text-foreground">Needs identification</p>{/if}
      {#if detail || badges}
        <div class="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
          {#if detail}<p>{detail}</p>{/if}
          {@render badges?.()}
        </div>
      {/if}
    </div>
  </a>
</li>
