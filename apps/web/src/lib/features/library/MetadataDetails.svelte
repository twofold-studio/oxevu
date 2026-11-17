<script lang="ts">
  import BuildingIcon from "@lucide/svelte/icons/building";
  import ExternalLinkIcon from "@lucide/svelte/icons/external-link";
  import TagsIcon from "@lucide/svelte/icons/tags";
  import UserRoundPenIcon from "@lucide/svelte/icons/user-round-pen";
  import type { Item } from "$lib/api/types";
  import { provider } from "$lib/media/metadata";
  import ProviderMark from "./metadata/ProviderMark.svelte";

  // Overview, credits and provider links. Years, certificate and score live in
  // the compact TitleFacts line under the page title.
  let { item, series = false }: { item: Item; series?: boolean } = $props();
  const overview = $derived(series ? item.show_overview : item.overview);
  const genres = $derived((series ? item.show_genres : item.genres) ?? []);
  const studios = $derived((series ? item.show_studios : item.studios) ?? []);
  const creators = $derived((series ? item.show_creators : item.creators) ?? []);
  // Movie matches store directors in `creators`; series store their creators.
  const creatorLabel = $derived(item.kind === "movie" && !series ? "Directed by" : "Created by");
  const studioImages = $derived((item.studio_images ?? []).filter((image) => /^\/api\/v1\/items\/[^/]+\/metadata\/studios\/[a-z0-9-]+$/.test(image.url)));

  function external(url: string | undefined, domain: string): string | null {
    if (!url) return null;
    try {
      const parsed = new URL(url);
      return parsed.protocol === "https:" && (parsed.hostname === domain || parsed.hostname === `www.${domain}`) ? parsed.href : null;
    } catch { return null; }
  }
  const links = $derived(
    [
      { source: provider("tmdb"), url: external(item.external_urls?.tmdb, "themoviedb.org") },
      { source: provider("imdb"), url: external(item.external_urls?.imdb, "imdb.com") },
    ].filter((link): link is { source: typeof link.source; url: string } => link.url !== null),
  );
</script>

{#if item.metadata_status === "needs_identification"}
  <div role="status" class="rounded-lg border bg-muted/40 px-4 py-3 text-sm">
    <p class="font-medium">Needs identification</p>
    <p class="mt-1 text-muted-foreground">{item.metadata_error ?? "Automatic metadata matching could not find a unique title."} Use Identify to search or paste a provider ID.</p>
  </div>
{:else if item.metadata_status === "pending"}
  <p role="status" class="text-sm text-muted-foreground">Metadata lookup queued…</p>
{/if}

{#if overview || genres.length || studios.length || creators.length || links.length}
  <section aria-label={series ? "Series information" : "Title information"} class="flex max-w-3xl flex-col gap-5">
    {#if overview}<p class="text-sm leading-7 text-muted-foreground">{overview}</p>{/if}
    {#if creators.length || studios.length || genres.length}
      <dl class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-2.5 text-sm leading-6">
        {#if creators.length}
          <dt title={creatorLabel} class="flex items-center text-muted-foreground"><UserRoundPenIcon class="size-4 opacity-70" aria-hidden="true" /><span class="sr-only">{creatorLabel}</span></dt>
          <dd>{creators.join(", ")}</dd>
        {/if}
        {#if studios.length}
          <dt title="Studios" class="flex items-center text-muted-foreground"><BuildingIcon class="size-4 opacity-70" aria-hidden="true" /><span class="sr-only">Studios</span></dt>
          <dd>{studios.join(", ")}</dd>
        {/if}
        {#if genres.length}
          <dt title="Genres" class="flex items-center text-muted-foreground"><TagsIcon class="size-4 opacity-70" aria-hidden="true" /><span class="sr-only">Genres</span></dt>
          <dd>
            <ul class="flex flex-wrap gap-1.5">
              {#each genres as genre (genre)}<li class="rounded-full border px-2 py-0.5 text-xs leading-5 text-foreground/85">{genre}</li>{/each}
            </ul>
          </dd>
        {/if}
      </dl>
    {/if}
    {#if studioImages.length}
      <div class="flex flex-wrap items-center gap-4">
        {#each studioImages as image (image.name)}<img src={image.url} alt={image.name} loading="lazy" onerror={(event) => { (event.currentTarget as HTMLImageElement).hidden = true; }} class="max-h-12 max-w-28 object-contain" />{/each}
      </div>
    {/if}
    {#if links.length}
      <ul aria-label="External pages" class="flex flex-wrap gap-2">
        {#each links as link (link.source.name)}
          <li>
            <a
              href={link.url}
              target="_blank"
              rel="noreferrer"
              aria-label={`Open on ${link.source.name} (new tab)`}
              class="inline-flex h-7 items-center gap-1.5 rounded-md border px-2 text-xs text-muted-foreground transition-colors duration-(--motion-control) ease-settle hover:bg-accent/50 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
            >
              <ProviderMark source={link.source} />
              <ExternalLinkIcon class="size-3" aria-hidden="true" />
            </a>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}
