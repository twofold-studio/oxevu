<script lang="ts">
  import { onMount } from "svelte";
  import PlayIcon from "@lucide/svelte/icons/play";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import CheckIcon from "@lucide/svelte/icons/check";
  import UsersRoundIcon from "@lucide/svelte/icons/users-round";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import * as Alert from "$lib/components/ui/alert";
  import Page from "$lib/components/Page.svelte";
  import DetailLayout from "$lib/components/DetailLayout.svelte";
  import NotFound from "$lib/components/NotFound.svelte";
  import { itemsApi } from "$lib/api/endpoints";
  import { ApiError, errorMessage } from "$lib/api/client";
  import type { Item } from "$lib/api/types";
  import { catalog } from "$lib/state/catalog.svelte";
  import { href } from "$lib/state/router.svelte";
  import { formatClock, formatRuntime } from "$lib/media/format";
  import { episodeNumberLabel, seasonKeyOf } from "$lib/media/grouping";
  import { resumePosition, watchedFraction } from "$lib/media/progress";
  import ProgressLine from "$lib/features/library/ProgressLine.svelte";
  import Artwork from "$lib/components/Artwork.svelte";
  import { artworkSrc } from "$lib/api/artwork";
  import PosterArt from "$lib/features/library/PosterArt.svelte";
  import IdentifyButton from "$lib/features/library/IdentifyButton.svelte";
  import MetadataDetails from "$lib/features/library/MetadataDetails.svelte";
  import TitleFacts from "$lib/features/library/metadata/TitleFacts.svelte";
  import MetaFact from "$lib/features/library/metadata/MetaFact.svelte";
  import HistoryIcon from "@lucide/svelte/icons/history";
  import FileFacts from "./FileFacts.svelte";
  import PlaybackNotice from "./PlaybackNotice.svelte";
  import CreatePartyDialog from "$lib/features/party/CreatePartyDialog.svelte";
  import { session } from "$lib/state/session.svelte";
  import { canCreateParties } from "$lib/state/access";

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
    if (item.kind === "episode" && artworkSrc(item.thumbnail_url))
      return { shape: "still" as const, src: item.thumbnail_url, title: item.title };
    if (artworkSrc(item.poster_url)) {
      const subject = item.kind === "episode" ? (item.show_title ?? item.title) : item.title;
      return { shape: "poster" as const, src: item.poster_url, title: subject };
    }
    return null;
  });

  const resume = $derived(item ? resumePosition(item) : null);
  const fraction = $derived(item ? watchedFraction(item) : null);

  // Neighbouring episodes within the same series, in season/episode order.
  const siblings = $derived.by(() => {
    if (!item || item.kind !== "episode") return null;
    const show = catalog.showOf(item);
    if (!show) return null;
    const ordered = show.seasons.flatMap((s) => s.episodes);
    const index = ordered.findIndex((e) => e.id === item.id);
    return { show, previous: ordered[index - 1], next: ordered[index + 1] };
  });

  let partyOpen = $state(false);
  let partyOpening = $state(0);
  const canParty = $derived(canCreateParties(session.user));

  const episodeLine = $derived(item?.kind === "episode" ? episodeNumberLabel(item) : null);
</script>

{#if item}
  {#snippet leadArt()}
    {#if art?.shape === "still"}
      <Artwork
        src={art.src}
        alt={`Still from ${art.title}`}
        eager
        class="aspect-video w-full max-w-xl rounded-xl border shadow-2xl shadow-black/60 lg:max-w-none"
      />
    {:else if art}
      <PosterArt src={art.src} title={art.title} />
    {/if}
  {/snippet}
  <DetailLayout
    backdrop={item.backdrop_url}
    poster={item.poster_url}
    art={art ? leadArt : undefined}
    artShape={art?.shape}
  >
    <div class="flex flex-col gap-6">
        <div class="flex flex-col gap-2">
          {#if item.kind === "episode"}
            <p class="text-sm text-muted-foreground">
              {#if siblings}
                <a href={href.show(siblings.show.slug, seasonKeyOf(item.season_number))} class="text-foreground hover:underline">{siblings.show.title}</a>
              {:else}
                {item.show_title ?? "Series"}
              {/if}
              {#if episodeLine}<span class="ml-2">{episodeLine}</span>{/if}
            </p>
          {/if}
          <h1
            class="text-4xl leading-[1.05] font-semibold tracking-tight text-balance break-words [text-shadow:0_2px_24px_rgb(0_0_0/0.6)] lg:text-5xl"
          >
            {item.title}
          </h1>
          <TitleFacts {item}>
            {#if item.completed}
              <MetaFact icon={CheckIcon}>Watched</MetaFact>
            {:else if resume !== null}
              <MetaFact icon={HistoryIcon}>
                Stopped at {formatClock(resume)}{item.duration_seconds
                  ? `, ${formatRuntime(item.duration_seconds - resume) ?? "<1m"} left`
                  : ""}
              </MetaFact>
            {/if}
          </TitleFacts>
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
          <IdentifyButton {item} onsaved={(updated) => (fetched = updated)} />
          {#if canParty}
            <Button
              size="lg"
              variant="outline"
              onclick={() => {
                partyOpening++;
                partyOpen = true;
              }}
            >
              <UsersRoundIcon data-icon="inline-start" />
              Watch together
            </Button>
          {/if}
        </div>

        <MetadataDetails {item} />

        <PlaybackNotice {item} />

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

    <div class="max-w-xl">
      <FileFacts {item} />
    </div>
  </DetailLayout>
  {#key partyOpening}
    {#if partyOpening > 0}<CreatePartyDialog bind:open={partyOpen} itemId={item.id} />{/if}
  {/key}
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
