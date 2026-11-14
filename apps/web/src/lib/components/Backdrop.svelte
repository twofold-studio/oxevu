<script lang="ts">
  import Artwork from "./Artwork.svelte";

  // Decorative wash behind a detail header: the real backdrop, dimmed and
  // feathered into the page so text above it keeps its contrast. Renders
  // nothing when there is no backdrop or it fails to load. The parent must be
  // `relative isolate`.
  let { src }: { src: string | null | undefined } = $props();
</script>

<div
  aria-hidden="true"
  class="pointer-events-none absolute inset-x-0 top-0 -z-10 h-[24rem] [mask-image:radial-gradient(ellipse_80%_100%_at_70%_0%,black_25%,transparent_75%)] opacity-30 lg:h-[28rem]"
>
  <Artwork {src} alt="" eager class="size-full bg-transparent" />
</div>
