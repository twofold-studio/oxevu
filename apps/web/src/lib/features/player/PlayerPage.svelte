<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { Spinner } from "$lib/components/ui/spinner";
  import { itemsApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import type { Item } from "$lib/api/types";
  import { catalog } from "$lib/state/catalog.svelte";
  import { session } from "$lib/state/session.svelte";
  import { href } from "$lib/state/router.svelte";
  import { resumePosition } from "$lib/media/progress";
  import { PlaybackEngine } from "$lib/playback/engine.svelte";
  import PlaybackFailure from "./PlaybackFailure.svelte";
  import PlayerView from "./PlayerView.svelte";

  let { id, fromStart }: { id: string; fromStart: boolean } = $props();

  let item = $state<Item | null>(null);
  let loadError = $state<string | null>(null);

  // One engine per fetched item; permission changes apply to the next session
  // server side, so they must not rebuild (and restart) the player.
  const engine = $derived.by(() => {
    const current = item;
    if (!current) return null;
    return untrack(
      () => new PlaybackEngine(current, session.user?.can_transcode ?? false, () => void session.load()),
    );
  });
  // Resume always uses the freshly fetched position, which may come from
  // another device.
  const start = $derived(item && !fromStart ? (resumePosition(item) ?? 0) : 0);

  onMount(() => {
    itemsApi
      .get(id)
      .then(({ item: fresh }) => {
        item = fresh;
        catalog.patch(id, fresh);
      })
      .catch((error: unknown) => (loadError = errorMessage(error)));
  });
</script>

<svelte:head>
  <title>{item ? `${item.title} · Oxevu` : "Oxevu"}</title>
</svelte:head>

{#if loadError}
  <div class="fixed inset-0 bg-black text-white">
    <PlaybackFailure title="Couldn't open this title" detail={loadError} backHref={href.home()} backLabel="Back to home" />
  </div>
{:else if engine}
  <PlayerView
    {engine}
    {start}
    backHref={href.item(engine.item.id)}
    canTranscode={session.user?.can_transcode ?? false}
  />
{:else}
  <div class="fixed inset-0 grid place-items-center bg-black" aria-busy="true">
    <Spinner class="size-8 text-white/80" />
  </div>
{/if}
