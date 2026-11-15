<script lang="ts">
  import type { RatingSource } from "$lib/media/metadata";
  import { cn } from "$lib/utils";

  // Typographic marks in each provider's published brand colours, drawn
  // locally rather than fetching or approximating official logo artwork.
  // TMDB: navy #0d253f with the green-to-blue #90cea1 → #01b4e4 lettering.
  // IMDb: black lettering on its #f5c518 yellow. Unknown sources stay plain
  // text, exactly as stored. The visible text is also the accessible name.
  let { source, class: className }: { source: RatingSource; class?: string } = $props();
</script>

{#if source.id === "tmdb"}
  <span class={cn("inline-flex h-4 items-center rounded-[3px] bg-[#0d253f] px-1 text-[0.625rem] leading-none font-bold tracking-wide", className)}>
    <span class="bg-linear-to-r from-[#90cea1] to-[#01b4e4] bg-clip-text text-transparent">{source.name}</span>
  </span>
{:else if source.id === "imdb"}
  <span class={cn("inline-flex h-4 items-center rounded-[3px] bg-[#f5c518] px-1 text-[0.625rem] leading-none font-extrabold text-black", className)}>
    {source.name}
  </span>
{:else}
  <span class={cn("inline-flex h-4 items-center rounded-[3px] border px-1 text-[0.625rem] leading-none font-medium text-muted-foreground", className)}>
    {source.name}
  </span>
{/if}
