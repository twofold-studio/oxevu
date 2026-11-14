<script lang="ts">
  import Artwork from "$lib/components/Artwork.svelte";

  // The lead poster on a detail page: compact beside the title on phones,
  // the full art column on desktop. If the image fails the title is set in
  // its place so the column never collapses into an empty gap.
  let { src, title }: { src: string | null | undefined; title: string } = $props();
</script>

<Artwork
  {src}
  alt={`Poster for ${title}`}
  eager
  class="aspect-[2/3] w-40 rounded-xl border bg-card shadow-2xl shadow-black/60 sm:w-52 lg:w-full"
>
  {#snippet fallback()}
    <span
      aria-hidden="true"
      class="absolute inset-0 flex items-end p-5 text-xl leading-tight font-semibold tracking-tight text-balance break-words text-foreground/80"
    >
      {title}
    </span>
  {/snippet}
</Artwork>
