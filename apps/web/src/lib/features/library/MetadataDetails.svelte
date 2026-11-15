<script lang="ts">
  import type { Item } from "$lib/api/types";

  let { item, series = false }: { item: Item; series?: boolean } = $props();
  const overview = $derived(series ? item.show_overview : item.overview);
  const year = $derived(series ? item.show_year : item.year);
  const endYear = $derived(series ? item.show_end_year : item.end_year);
  const contentRating = $derived(series ? item.show_content_rating : item.content_rating);
  const rating = $derived(series ? item.show_rating : item.rating);
  const genres = $derived((series ? item.show_genres : item.genres) ?? []);
  const studios = $derived((series ? item.show_studios : item.studios) ?? []);
  const creators = $derived((series ? item.show_creators : item.creators) ?? []);
  const studioImages = $derived((item.studio_images ?? []).filter((image) => /^\/api\/v1\/items\/[^/]+\/metadata\/studios\/[a-z0-9-]+$/.test(image.url)));

  function external(url: string | undefined, domain: string): string | null {
    if (!url) return null;
    try {
      const parsed = new URL(url);
      return parsed.protocol === "https:" && (parsed.hostname === domain || parsed.hostname === `www.${domain}`) ? parsed.href : null;
    } catch { return null; }
  }
  const tmdb = $derived(external(item.external_urls?.tmdb, "themoviedb.org"));
  const imdb = $derived(external(item.external_urls?.imdb, "imdb.com"));
</script>

{#if overview || year || rating || genres.length || studios.length || creators.length || tmdb || imdb}
  <section aria-label={series ? "Series information" : "Title information"} class="flex max-w-3xl flex-col gap-4">
    {#if year || contentRating || rating}
      <p class="flex flex-wrap gap-3 text-sm text-muted-foreground">
        {#if year}<span>{year}{endYear && endYear !== year ? `–${endYear}` : ""}</span>{/if}
        {#if contentRating}<span>{contentRating}</span>{/if}
        {#if rating}<span>Rating {rating.toFixed(1)}/10</span>{/if}
      </p>
    {/if}
    {#if overview}<p class="text-sm leading-7 text-muted-foreground">{overview}</p>{/if}
    {#if creators.length || studios.length || genres.length}
      <dl class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-5 gap-y-2 text-sm">
        {#if creators.length}<dt class="text-muted-foreground">Creators</dt><dd>{creators.join(", ")}</dd>{/if}
        {#if studios.length}<dt class="text-muted-foreground">Studios</dt><dd>{studios.join(", ")}</dd>{/if}
        {#if genres.length}<dt class="text-muted-foreground">Genres</dt><dd>{genres.join(", ")}</dd>{/if}
      </dl>
    {/if}
    {#if studioImages.length}
      <div class="flex flex-wrap items-center gap-4">
        {#each studioImages as image (image.name)}<img src={image.url} alt={image.name} loading="lazy" onerror={(event) => { (event.currentTarget as HTMLImageElement).hidden = true; }} class="max-h-12 max-w-28 object-contain" />{/each}
      </div>
    {/if}
    {#if tmdb || imdb}
      <p class="flex gap-4 text-sm">
        {#if tmdb}<a href={tmdb} target="_blank" rel="noreferrer" class="underline underline-offset-4">TMDB</a>{/if}
        {#if imdb}<a href={imdb} target="_blank" rel="noreferrer" class="underline underline-offset-4">IMDb</a>{/if}
      </p>
    {/if}
  </section>
{/if}
