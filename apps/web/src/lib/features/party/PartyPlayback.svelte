<script lang="ts">
  import { onMount } from "svelte";
  import { Spinner } from "$lib/components/ui/spinner";
  import { partySession } from "$lib/party/session.svelte";
  import { session } from "$lib/state/session.svelte";
  import { href, router } from "$lib/state/router.svelte";
  import { errorMessage } from "$lib/api/client";
  import PlaybackFailure from "$lib/features/player/PlaybackFailure.svelte";
  import PartyPlayer from "./PartyPlayer.svelte";

  let { id, fromStart }: { id: string; fromStart: boolean } = $props();
  let failure = $state<string | null>(null);
  const connection = $derived(partySession.connection);
  const item = $derived(connection?.item);

  onMount(() => {
    void partySession.play(id, fromStart).catch((error) => (failure = errorMessage(error)));
  });
</script>

{#if failure || connection?.itemError}
  <div class="fixed inset-0 bg-black text-white">
    <PlaybackFailure title="Couldn't play for the party" detail={failure ?? connection?.itemError ?? ""} backHref={href.parties()} backLabel="Back to parties" />
  </div>
{:else if connection && item && item.id === id && connection.party?.item_id === id}
  <PartyPlayer {item} {connection} isHost={connection.party.host_user_id === session.user?.id} onleave={() => { partySession.leave(); router.go(href.parties()); }} />
{:else}
  <div class="fixed inset-0 grid place-items-center bg-black text-white" role="status"><Spinner class="size-8" /><span class="sr-only">Opening party playback</span></div>
{/if}
