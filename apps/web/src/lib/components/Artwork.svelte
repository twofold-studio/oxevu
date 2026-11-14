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
        onerror={() => (failed = url)}
        class={cn("absolute inset-0 size-full object-cover", imageClass)}
      />
    {:else}
      {@render fallback?.()}
    {/if}
  </div>
{/if}
