<script lang="ts">
  import type { Snippet } from "svelte";
  import { artworkSrc } from "$lib/api/artwork";
  import { cn } from "$lib/utils";
  import Backdrop from "./Backdrop.svelte";
  import Page from "./Page.svelte";

  // Shared frame for movie, episode and series pages: a full-width artwork
  // band, then a compact hero (lead artwork beside the title, facts, actions
  // and overview), then optional full-width content such as a series' season
  // tabs and episodes. Phones stack everything in one column. `art` is
  // "poster" (2:3) or "still" (16:9, wider column); it only sizes the column,
  // the caller renders the image.
  //
  // `aside` is an optional third hero column shown only from xl, where there
  // is room beside the title (a series' Continue watching card). It must only
  // repeat what the hero already offers, because narrower screens omit it.
  // `below` spans the full page width under the hero, so long lists are not
  // squeezed beside a poster column that has nothing left to show.
  let {
    backdrop,
    poster = null,
    art,
    artShape = "poster",
    aside,
    below,
    children,
  }: {
    backdrop: string | null | undefined;
    poster?: string | null;
    art?: Snippet;
    artShape?: "poster" | "still";
    aside?: Snippet;
    below?: Snippet;
    children: Snippet;
  } = $props();

  // Only pull content up into the band when there is art to sit over.
  const banded = $derived(artworkSrc(backdrop) !== null || artworkSrc(poster) !== null);
</script>

<div class="relative isolate">
  <Backdrop src={backdrop} {poster} />
  <Page class={cn(banded && "pt-[clamp(5rem,16vw,11rem)] lg:pt-[clamp(5rem,14vw,11rem)]")}>
    <div
      class={cn(
        "grid gap-6 sm:gap-8 lg:gap-10",
        art && artShape === "poster" && "lg:grid-cols-[clamp(220px,22vw,300px)_minmax(0,1fr)]",
        art && artShape === "still" && "lg:grid-cols-[clamp(340px,38vw,560px)_minmax(0,1fr)]",
        aside && art && artShape === "poster" && "xl:grid-cols-[clamp(220px,18vw,280px)_minmax(0,1fr)_clamp(18rem,24vw,22rem)]",
        aside && !art && "xl:grid-cols-[minmax(0,1fr)_clamp(18rem,26vw,22rem)]",
      )}
    >
      {#if art}
        <div class="min-w-0 lg:self-start">{@render art()}</div>
      {/if}
      <div class="flex min-w-0 flex-col gap-6">{@render children()}</div>
      {#if aside}
        <div class="hidden min-w-0 flex-col gap-6 xl:flex">{@render aside()}</div>
      {/if}
    </div>
    {#if below}
      <div class="mt-10 flex flex-col gap-8 lg:mt-14">{@render below()}</div>
    {/if}
  </Page>
</div>
