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
  //
  // `aside` is an optional secondary column (a series' Continue watching and
  // seasons): beside the main column on wide screens so it shares the first
  // viewport with the title, under the main column on narrower ones. Pages
  // with an aside start closer to the top of the band, since the band only
  // needs to frame the title, not hold the page's content alone.
  let {
    backdrop,
    poster = null,
    art,
    artShape = "poster",
    aside,
    children,
  }: {
    backdrop: string | null | undefined;
    poster?: string | null;
    art?: Snippet;
    artShape?: "poster" | "still";
    aside?: Snippet;
    children: Snippet;
  } = $props();

  // Only pull content up into the band when there is art to sit over.
  const banded = $derived(artworkSrc(backdrop) !== null || artworkSrc(poster) !== null);
</script>

<!-- Reserve the small viewport below the 4rem desktop header, but let long
     descriptions and season shelves extend the page instead of clipping. -->
<div class={cn("relative isolate", aside && "xl:min-h-[calc(100svh-4rem)]")}>
  <Backdrop src={backdrop} {poster} />
  <Page
    class={cn(
      banded && !aside && "pt-[clamp(6rem,20vw,15rem)] lg:pt-[clamp(6rem,17vw,15rem)]",
      banded && aside && "pt-[clamp(5rem,16vw,8rem)] lg:pt-[clamp(2rem,7svh,8rem)]",
    )}
  >
    <div
      class={cn(
        "grid gap-8 lg:gap-12",
        !aside && art && artShape === "poster" && "lg:grid-cols-[clamp(260px,24vw,340px)_minmax(0,1fr)]",
        !aside && art && artShape === "still" && "lg:grid-cols-[clamp(340px,38vw,560px)_minmax(0,1fr)]",
        // With an aside the poster column narrows so three columns fit from xl.
        aside && art && "lg:grid-cols-[clamp(220px,20vw,300px)_minmax(0,1fr)] xl:grid-cols-[clamp(220px,18vw,300px)_minmax(0,1fr)_clamp(18rem,24vw,24rem)] xl:gap-x-10",
        aside && !art && "xl:grid-cols-[minmax(0,1fr)_clamp(18rem,28vw,26rem)] xl:gap-x-10",
      )}
    >
      {#if art}
        <div class={cn("min-w-0 lg:sticky lg:top-24 lg:self-start", aside && "lg:row-span-2 xl:row-span-1")}>{@render art()}</div>
      {/if}
      <div class="flex min-w-0 flex-col gap-8">{@render children()}</div>
      {#if aside}
        <div class={cn("flex min-w-0 flex-col gap-8 xl:gap-6", art && "lg:col-start-2 xl:col-start-3")}>{@render aside()}</div>
      {/if}
    </div>
  </Page>
</div>
