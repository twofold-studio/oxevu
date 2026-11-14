<script lang="ts">
  import type { Snippet } from "svelte";
  import { artworkSrc } from "$lib/api/artwork";
  import { cn } from "$lib/utils";

  // One framed image with a fixed aspect ratio set by `class`, so layout never
  // shifts while it loads. A missing or failed image shows `fallback` instead
  // of the browser's broken-image icon; without a fallback the frame is
  // dropped entirely (used for optional detail art).
  let {
    src,
    alt,
    class: className,
    imageClass,
    eager = false,
    fallback,
  }: {
    src: string | null | undefined;
    /** Empty when the image sits beside text that already names it. */
    alt: string;
    class?: string;
    imageClass?: string;
    eager?: boolean;
    fallback?: Snippet;
  } = $props();

  const url = $derived(artworkSrc(src));
  // Keyed by URL so a later record with different artwork gets a fresh try.
  let failed = $state<string | null>(null);
  const visible = $derived(url !== null && failed !== url);

  // Reveal state, keyed by URL like `failed`: "fresh" fades in on load,
  // "cached" (already complete when mounted, e.g. returning to a page) shows
  // at once. The frame's muted fill holds the space meanwhile.
  let loaded = $state<{ url: string; how: "fresh" | "cached" } | null>(null);
  const loadedAs = $derived(loaded && loaded.url === url ? loaded.how : undefined);

  function markLoaded(how: "fresh" | "cached") {
    if (url !== null && loaded?.url !== url) loaded = { url, how };
  }

  // Runs again when `url` changes; a load event may already have fired
  // before listeners existed, so check `complete` directly.
  function checkCached(img: HTMLImageElement) {
    if (url !== null && img.complete && img.naturalWidth > 0) markLoaded("cached");
  }
</script>

{#if visible || fallback}
  <div class={cn("relative overflow-hidden bg-muted", className)}>
    {#if visible}
      <img
        src={url}
        {alt}
        loading={eager ? "eager" : "lazy"}
        decoding="async"
        draggable="false"
        data-loaded={loadedAs}
        onload={() => markLoaded("fresh")}
        onerror={() => (failed = url)}
        {@attach checkCached}
        class={cn("reveal absolute inset-0 size-full object-cover", imageClass)}
      />
    {:else}
      {@render fallback?.()}
    {/if}
  </div>
{/if}
