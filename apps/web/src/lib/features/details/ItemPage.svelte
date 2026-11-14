<script lang="ts">
  import { onMount } from "svelte";
  import PlayIcon from "@lucide/svelte/icons/play";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import CheckIcon from "@lucide/svelte/icons/check";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import * as Alert from "$lib/components/ui/alert";
  import Page from "$lib/components/Page.svelte";
  import NotFound from "$lib/components/NotFound.svelte";
  import { itemsApi } from "$lib/api/endpoints";
  import { ApiError, errorMessage } from "$lib/api/client";
  import type { Item } from "$lib/api/types";
  import { catalog } from "$lib/state/catalog.svelte";
  import { href } from "$lib/state/router.svelte";
  import { showKey } from "$lib/media/grouping";
  import { formatClock, formatRuntime } from "$lib/media/format";
  import { resumePosition, watchedFraction } from "$lib/media/progress";
  import ProgressLine from "$lib/features/library/ProgressLine.svelte";
  import Artwork from "$lib/components/Artwork.svelte";
  import Backdrop from "$lib/components/Backdrop.svelte";
  import FileFacts from "./FileFacts.svelte";
  import PlaybackNotice from "./PlaybackNotice.svelte";

  let { id }: { id: string } = $props();

  // Show the cached copy immediately, then refresh from the server so
  // progress written from another device is current.
  let fetched = $state<Item | null>(null);
  let error = $state<{ missing: boolean; message: string } | null>(null);
  const item = $derived(fetched ?? catalog.find(id) ?? null);

  onMount(() => {
    void catalog.ensure();
    itemsApi
      .get(id)
      .then(({ item }) => {
        fetched = item;
        catalog.patch(id, item);
      })
      .catch((e: unknown) => {
        error = { missing: e instanceof ApiError && e.status === 404, message: errorMessage(e) };
      });
  });

  // Movies lead with their poster; episodes with their own still, else the
  // show poster they inherit. Old records without artwork fields show none.
  const art = $derived.by(() => {
    if (!item) return null;
    if (item.kind === "episode" && item.thumbnail_url)
      return { src: item.thumbnail_url, alt: `Still from ${item.title}`, frame: "aspect-video w-full max-w-md" };
    if (item.poster_url) {
      const subject = item.kind === "episode" ? (item.show_title ?? item.title) : item.title;
      return { src: item.poster_url, alt: `Poster for ${subject}`, frame: "aspect-[2/3] w-32 sm:w-44" };
    }
    return null;
  });

  const resume = $derived(item ? resumePosition(item) : null);
  const fraction = $derived(item ? watchedFraction(item) : null);

  // Neighbouring episodes within the same series, in season/episode order.
  const siblings = $derived.by(() => {
    if (!item || item.kind !== "episode") return null;
    const show = catalog.shows.find((s) => s.key === showKey(item));
    if (!show) return null;
    const ordered = show.seasons.flatMap((s) => s.episodes);
    const index = ordered.findIndex((e) => e.id === item.id);
    return { show, previous: ordered[index - 1], next: ordered[index + 1] };
  });

  const episodeLine = $derived.by(() => {
    if (!item || item.kind !== "episode") return null;
    const parts = [];
    if (item.season_number !== null) parts.push(item.season_number === 0 ? "Special" : `Season ${item.season_number}`);
    if (item.episode_number !== null) parts.push(`Episode ${item.episode_number}`);
    return parts.join(", ") || null;
  });
</script>

{#if item}
  <Page class="relative isolate">
    <Backdrop src={item.backdrop_url} />
    <div class="grid gap-10 lg:grid-cols-[minmax(0,1fr)_18rem]">
      <div class="flex min-w-0 flex-col gap-6">
        {#if art}
          <Artwork
            src={art.src}
            alt={art.alt}
            eager
            class={`${art.frame} shrink-0 rounded-lg border shadow-lg shadow-black/40`}
          />
        {/if}
        <div class="flex flex-col gap-2">
          {#if item.kind === "episode"}
            <p class="text-sm text-muted-foreground">
              {#if siblings}
                <a href={href.show(siblings.show.key)} class="text-foreground hover:underline">{siblings.show.title}</a>
              {:else}
                {item.show_title ?? "Series"}
              {/if}
              {#if episodeLine}<span class="ml-2">{episodeLine}</span>{/if}
            </p>
          {/if}
          <h1 class="text-3xl leading-tight font-semibold tracking-tight text-balance break-words sm:text-4xl">
            {item.title}
          </h1>
          <p class="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-muted-foreground">
            {#if formatRuntime(item.duration_seconds)}<span>{formatRuntime(item.duration_seconds)}</span>{/if}
            {#if item.completed}
              <span class="flex items-center gap-1.5"><CheckIcon class="size-3.5" /> Watched</span>
            {:else if resume !== null}
              <span>
                Stopped at {formatClock(resume)}{item.duration_seconds
                  ? `, ${formatRuntime(item.duration_seconds - resume) ?? "<1m"} left`
                  : ""}
              </span>
            {/if}
          </p>
        </div>

        {#if fraction !== null && !item.completed}
          <ProgressLine value={fraction} class="max-w-md rounded-full" />
        {/if}

        <div class="flex flex-wrap gap-2">
          <Button href={href.watch(item.id)} size="lg">
            <PlayIcon data-icon="inline-start" />
            {resume !== null ? `Resume from ${formatClock(resume)}` : item.completed ? "Play again" : "Play"}
          </Button>
          {#if resume !== null}
            <Button href={href.watch(item.id, true)} size="lg" variant="outline">
              <RotateCcwIcon data-icon="inline-start" />
              Start over
            </Button>
          {/if}
        </div>

        <div class="max-w-2xl">
          <PlaybackNotice {item} />
        </div>

        {#if error && !error.missing}
          <Alert.Root>
            <Alert.Description>Showing saved details. Refresh failed: {error.message}</Alert.Description>
          </Alert.Root>
        {/if}

        {#if siblings && (siblings.previous || siblings.next)}
          <nav aria-label="Episodes" class="mt-4 grid gap-2 border-t pt-6 sm:grid-cols-2">
            {#if siblings.previous}
              <a
                href={href.item(siblings.previous.id)}
                class="flex items-center gap-3 rounded-lg border px-4 py-3 transition-colors hover:bg-accent/40"
              >
                <ChevronLeftIcon class="size-4 shrink-0 text-muted-foreground" />
                <span class="flex min-w-0 flex-col">
                  <span class="text-xs text-muted-foreground">Previous</span>
                  <span class="truncate text-sm font-medium">{siblings.previous.title}</span>
                </span>
              </a>
            {:else}
              <span class="hidden sm:block"></span>
            {/if}
            {#if siblings.next}
              <a
                href={href.item(siblings.next.id)}
                class="flex items-center justify-end gap-3 rounded-lg border px-4 py-3 text-right transition-colors hover:bg-accent/40"
              >
                <span class="flex min-w-0 flex-col">
                  <span class="text-xs text-muted-foreground">Next</span>
                  <span class="truncate text-sm font-medium">{siblings.next.title}</span>
                </span>
                <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
              </a>
            {/if}
          </nav>
        {/if}
      </div>

      <aside class="lg:border-l lg:pl-8">
        <FileFacts {item} />
      </aside>
    </div>
  </Page>
{:else if error}
  {#if error.missing}
    <NotFound title="Title not found" description="It may have been removed by a rescan." />
  {:else}
    <Page>
      <Alert.Root variant="destructive">
        <Alert.Title>Couldn't load this title</Alert.Title>
        <Alert.Description>{error.message}</Alert.Description>
      </Alert.Root>
    </Page>
  {/if}
{:else}
  <Page>
    <div class="flex flex-col gap-4" aria-busy="true">
      <Skeleton class="h-4 w-32" />
      <Skeleton class="h-10 w-2/3" />
      <Skeleton class="h-4 w-24" />
      <Skeleton class="mt-2 h-9 w-40" />
    </div>
  </Page>
{/if}
