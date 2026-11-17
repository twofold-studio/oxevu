<script lang="ts">
  import { onMount } from "svelte";
  import PlayIcon from "@lucide/svelte/icons/play";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import CheckIcon from "@lucide/svelte/icons/check";
  import UsersRoundIcon from "@lucide/svelte/icons/users-round";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import HardDriveOffIcon from "@lucide/svelte/icons/hard-drive";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import * as Alert from "$lib/components/ui/alert";
  import Page from "$lib/components/Page.svelte";
  import DetailLayout from "$lib/components/DetailLayout.svelte";
  import NotFound from "$lib/components/NotFound.svelte";
  import { itemsApi } from "$lib/api/endpoints";
  import type { RemovalResult } from "$lib/api/endpoints";
  import { ApiError, errorMessage } from "$lib/api/client";
  import type { Item } from "$lib/api/types";
  import { catalog } from "$lib/state/catalog.svelte";
  import { fileAvailable, itemPlayable, catalogVisibleRefresh } from "$lib/state/catalog.svelte";
  import { href, router } from "$lib/state/router.svelte";
  import { formatClock, formatRuntime } from "$lib/media/format";
  import { episodeNumberLabel, seasonKeyOf, seasonLabel } from "$lib/media/grouping";
  import { resumePosition, watchedFraction, withWatchedStatus } from "$lib/media/progress";
  import ProgressLine from "$lib/features/library/ProgressLine.svelte";
  import Artwork from "$lib/components/Artwork.svelte";
  import { artworkSrc } from "$lib/api/artwork";
  import PosterArt from "$lib/features/library/PosterArt.svelte";
  import IdentifyButton from "$lib/features/library/IdentifyButton.svelte";
  import MetadataDetails from "$lib/features/library/MetadataDetails.svelte";
  import TitleFacts from "$lib/features/library/metadata/TitleFacts.svelte";
  import MetaFact from "$lib/features/library/metadata/MetaFact.svelte";
  import HistoryIcon from "@lucide/svelte/icons/history";
  import PlaybackNotice from "./PlaybackNotice.svelte";
  import FavoriteButton from "$lib/features/library/FavoriteButton.svelte";
  import TrailerButton from "./TrailerButton.svelte";
  import DownloadFileButton from "$lib/features/library/DownloadFileButton.svelte";
  import FindAgainLink from "./FindAgainLink.svelte";
  import RemovalDialog from "./RemovalDialog.svelte";
  import { partySession } from "$lib/party/session.svelte";
  import DownloadBadge from "$lib/features/library/DownloadBadge.svelte";
  import { session } from "$lib/state/session.svelte";
  import { canJoinParties, isAdmin, canManageDownloads } from "$lib/state/access";

  let { id }: { id: string } = $props();

  // Show the cached copy immediately, then refresh from the server so
  // progress written from another device is current.
  let fetched = $state<Item | null>(null);
  let error = $state<{ missing: boolean; message: string } | null>(null);
  const item = $derived(catalog.find(id) ?? fetched);

  onMount(() => {
    const controller = new AbortController();
    const stopRefresh = catalogVisibleRefresh();
    void catalog.ensure();
    itemsApi
      .get(id, controller.signal)
      .then(({ item }) => {
        if (controller.signal.aborted) return;
        fetched = withWatchedStatus(item);
        catalog.patch(id, item);
      })
      .catch((e: unknown) => {
        if (controller.signal.aborted) return;
        error = { missing: e instanceof ApiError && e.status === 404, message: errorMessage(e) };
      });
    return () => { controller.abort(); stopRefresh(); };
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

  const hasFile = $derived(item ? fileAvailable(item) : true);
  const playable = $derived(item ? itemPlayable(item) : true);
  const admin = $derived(isAdmin(session.user));
  const canDownload = $derived(canManageDownloads(session.user));

  // Neighbouring episodes within the same series, in season/episode order.
  const siblings = $derived.by(() => {
    if (!item || item.kind !== "episode") return null;
    const show = catalog.showOf(item);
    if (!show) return null;
    const ordered = show.seasons.flatMap((s) => s.episodes);
    const index = ordered.findIndex((e) => e.id === item.id);
    return { show, previous: ordered[index - 1], next: ordered[index + 1] };
  });

  const canParty = $derived(canJoinParties(session.user) && session.user?.can_play === true && hasFile
    && partySession.connection?.state === "live"
    && partySession.connection.party?.members.some((member) => member.user_id === session.user?.id));

  let removalOpen = $state(false);

  const episodeLine = $derived(item?.kind === "episode" ? episodeNumberLabel(item) : null);

  function handleRemoved(r: RemovalResult) {
    if (!r.kept_metadata && !r.failures.length && r.removed_ids.includes(id)) {
      router.go(href.home());
    } else {
      fetched = catalog.find(id) ?? fetched;
    }
  }
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
  <!-- Previous/next episode in series order, full width under the hero. -->
  {#snippet episodeNav()}
    {#if siblings}
      <nav aria-label="Episodes" class="grid gap-2 border-t pt-6 sm:grid-cols-2">
        {#if siblings.previous}
          <a
            href={href.item(siblings.previous.id)}
            class="flex items-center gap-3 rounded-lg border px-4 py-3 transition-colors hover:bg-accent/40"
          >
            <ChevronLeftIcon class="size-4 shrink-0 text-muted-foreground" />
            <span class="flex min-w-0 flex-col">
              <span class="text-xs text-muted-foreground">Previous · {episodeNumberLabel(siblings.previous) ?? "Episode"}</span>
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
              <span class="text-xs text-muted-foreground">Next · {episodeNumberLabel(siblings.next) ?? "Episode"}</span>
              <span class="truncate text-sm font-medium">{siblings.next.title}</span>
            </span>
            <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
          </a>
        {/if}
      </nav>
    {/if}
  {/snippet}
  <DetailLayout
    backdrop={item.backdrop_url}
    poster={item.poster_url}
    art={art ? leadArt : undefined}
    artShape={art?.shape}
    below={siblings && (siblings.previous || siblings.next) ? episodeNav : undefined}
  >
    <div class="flex flex-col gap-6">
        <div class="flex flex-col gap-2">
          {#if item.kind === "episode"}
            <!-- Series › Season › Episode, the same hierarchy as the show page;
                 the season link opens that season's tab there. -->
            <p class="flex flex-wrap items-center gap-x-1.5 gap-y-1 text-sm text-muted-foreground">
              {#if siblings}
                <a href={href.show(siblings.show.slug)} class="font-medium text-foreground hover:underline">{siblings.show.title}</a>
                <ChevronRightIcon aria-hidden="true" class="size-3.5 opacity-60" />
                <a href={href.show(siblings.show.slug, seasonKeyOf(item.season_number))} class="hover:text-foreground hover:underline">{seasonLabel(item.season_number)}</a>
                {@const number = episodeNumberLabel(item, { withSeason: false })}
                {#if number}<ChevronRightIcon aria-hidden="true" class="size-3.5 opacity-60" /><span>{number}</span>{/if}
              {:else}
                <span class="text-foreground">{item.show_title ?? "Series"}</span>
                {#if episodeLine}<span class="ml-1">{episodeLine}</span>{/if}
              {/if}
            </p>
          {/if}
          <h1
            class="text-3xl leading-[1.05] font-semibold tracking-tight text-balance break-words [text-shadow:0_2px_24px_rgb(0_0_0/0.6)] sm:text-4xl lg:text-5xl"
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

        {#if !hasFile && item.download}
          <div class="flex items-center gap-2">
            <DownloadBadge download={item.download} />
          </div>
        {:else if !hasFile}
          <div class="flex items-center gap-2">
            <Badge variant="outline">
              <HardDriveOffIcon data-icon="inline-start" />
              No local file
            </Badge>
          </div>
        {/if}

        <div class="flex flex-wrap gap-2">
          {#if playable}
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
          {/if}
          <IdentifyButton {item} onsaved={(updated) => (fetched = updated)} />
          <FavoriteButton items={[item]} />
          <DownloadFileButton {item} />
          {#if item.kind === "movie"}
            <TrailerButton kind="movie" providerId={item.provider_ids?.tmdb} title={item.title} />
          {/if}
          {#if canParty}
            <Button
              size="lg"
              variant="outline"
              href={href.watch(item.id)}
            >
              <UsersRoundIcon data-icon="inline-start" />
              Watch together
            </Button>
          {/if}
          {#if admin && !item.download}
            <Button
              size="lg"
              variant="outline"
              onclick={() => (removalOpen = true)}
            >
              <Trash2Icon data-icon="inline-start" />
              Remove
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

        {#if !hasFile && !item.download && canDownload}
          <FindAgainLink {item} />
        {/if}

    </div>

  </DetailLayout>
  {#if admin}
    <RemovalDialog bind:open={removalOpen} itemIds={[item.id]} label={item.title} onremoved={handleRemoved} />
  {/if}
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
