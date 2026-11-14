<script lang="ts">
  import type { Snippet } from "svelte";
  import { artworkSrc } from "$lib/api/artwork";
  import { cn } from "$lib/utils";
  import Backdrop from "./Backdrop.svelte";
  import Page from "./Page.svelte";

  // Shared frame for title detail pages: a full-width artwork band, then the
  // lead artwork in its own column with the title, actions and lists beside
  // it. Phones stack everything in one column. `art` is "poster" (2:3) or
  // "still" (16:9, wider column); it only sizes the column, the caller
  // renders the image.
  let {
    backdrop,
    poster = null,
    art,
    artShape = "poster",
    children,
  }: {
    backdrop: string | null | undefined;
    poster?: string | null;
    art?: Snippet;
    artShape?: "poster" | "still";
    children: Snippet;
  } = $props();

  // Only pull content up into the band when there is art to sit over.
  const banded = $derived(artworkSrc(backdrop) !== null || artworkSrc(poster) !== null);
</script>

<div class="relative isolate">
  <Backdrop src={backdrop} {poster} />
  <Page class={cn(banded && "pt-[clamp(6rem,20vw,15rem)] lg:pt-[clamp(6rem,17vw,15rem)]")}>
    <div
      class={cn(
        "grid gap-8 lg:gap-12",
        art && artShape === "poster" && "lg:grid-cols-[clamp(260px,24vw,340px)_minmax(0,1fr)]",
        art && artShape === "still" && "lg:grid-cols-[clamp(340px,38vw,560px)_minmax(0,1fr)]",
      )}
    >
      {#if art}
        <div class="min-w-0 lg:sticky lg:top-24 lg:self-start">{@render art()}</div>
      {/if}
      <div class="flex min-w-0 flex-col gap-8">{@render children()}</div>
    </div>
  </Page>
</div>
