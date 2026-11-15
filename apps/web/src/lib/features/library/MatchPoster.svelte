<script lang="ts">
  import FilmIcon from "@lucide/svelte/icons/film";
  let { url }: { url: string | null } = $props();
  let failed = $state(false);
  const source = $derived.by(() => {
    if (!url) return null;
    try {
      const parsed = new URL(url);
      return parsed.protocol === "https:" && !parsed.username && !parsed.password &&
        ["image.tmdb.org", "m.media-amazon.com", "ia.media-imdb.com"].includes(parsed.hostname)
        ? parsed.href : null;
    } catch { return null; }
  });
</script>

<span class="relative block aspect-[2/3] w-20 shrink-0 overflow-hidden rounded-md border bg-muted sm:w-24">
  {#if source && !failed}
    <img src={source} alt="" loading="lazy" referrerpolicy="no-referrer" class="size-full object-cover" onerror={() => (failed = true)} />
  {:else}
    <span class="absolute inset-0 grid place-items-center text-muted-foreground" aria-hidden="true"><FilmIcon class="size-6" /></span>
  {/if}
</span>
