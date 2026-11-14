<script lang="ts">
  import Artwork from "./Artwork.svelte";

  // Full-bleed artwork band behind a detail page header. Uses the real
  // backdrop; without one (or if it fails) the poster is blurred into a wash
  // instead, so no art is invented. Gradients feather it into the page so text
  // laid over its lower half keeps contrast. The parent must be
  // `relative isolate` and span the full content width.
  let { src, poster = null }: { src: string | null | undefined; poster?: string | null } = $props();
</script>

<div
  aria-hidden="true"
  class="pointer-events-none absolute inset-x-0 top-0 -z-10 h-[clamp(15rem,42vw,30rem)] overflow-hidden"
>
  <Artwork {src} alt="" eager class="size-full bg-transparent" imageClass="object-[50%_25%] opacity-60">
    {#snippet fallback()}
      <Artwork
        src={poster}
        alt=""
        eager
        class="absolute inset-0 bg-transparent"
        imageClass="scale-125 opacity-35 blur-3xl saturate-150"
      />
    {/snippet}
  </Artwork>
  <div class="absolute inset-0 bg-gradient-to-t from-background via-background/70 to-background/10"></div>
  <div class="absolute inset-0 bg-gradient-to-r from-background/70 via-transparent to-transparent"></div>
</div>
