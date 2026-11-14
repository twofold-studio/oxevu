<script lang="ts">
  import { onMount } from "svelte";
  import { Spinner } from "$lib/components/ui/spinner";
  import { session } from "$lib/state/session.svelte";
  import { href, router } from "$lib/state/router.svelte";
  import { PartyConnection } from "$lib/party/connection.svelte";
  import PlaybackFailure from "$lib/features/player/PlaybackFailure.svelte";
  import PartyPlayer from "./PartyPlayer.svelte";

  let { id }: { id: string } = $props();

  // svelte-ignore state_referenced_locally
  const connection = new PartyConnection(id);
  const item = $derived(connection.item);
  const itemError = $derived(connection.itemError);
  const party = $derived(connection.party);
  const isHost = $derived(party !== null && party.host_user_id === session.user?.id);
  const ended = $derived(connection.state === "closed" || connection.state === "failed");

  onMount(() => {
    void connection.start();
    // Refresh or tab close: stop polling but stay a member, so a host's
    // reload doesn't end the party. Idle members are reaped by the server.
    const onPageHide = () => connection.stop();
    window.addEventListener("pagehide", onPageHide);
    return () => {
      window.removeEventListener("pagehide", onPageHide);
      // Navigating away inside the app is an explicit leave.
      if (!ended) connection.leave(true);
    };
  });

  function leave() {
    connection.leave();
    router.go(href.parties());
  }
</script>

<svelte:head>
  <title>{party ? `${party.name} · Oxevu` : "Watch party · Oxevu"}</title>
</svelte:head>

{#if ended}
  <div class="fixed inset-0 bg-black text-white">
    <PlaybackFailure
      title={connection.state === "closed" ? "This party is over" : "Couldn't join the party"}
      detail={connection.message ?? ""}
      backHref={href.parties()}
      backLabel="Back to parties"
    />
  </div>
{:else if itemError}
  <div class="fixed inset-0 bg-black text-white">
    <PlaybackFailure title="Couldn't open the party's title" detail={itemError} backHref={href.parties()} backLabel="Back to parties" />
  </div>
{:else if item && party && item.id === party.item_id}
  {#key item.id}
    <PartyPlayer {item} {connection} {isHost} onleave={leave} />
  {/key}
{:else}
  <div class="fixed inset-0 grid place-items-center bg-black text-white" aria-busy="true">
    <div class="flex flex-col items-center gap-3">
      <Spinner class="size-8 text-white/80" />
      <span class="text-sm text-white/70">{connection.state === "joining" ? "Joining party" : "Loading title"}</span>
    </div>
  </div>
{/if}
