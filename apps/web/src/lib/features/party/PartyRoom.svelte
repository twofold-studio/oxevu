<script lang="ts">
  import { onMount } from "svelte";
  import UsersRoundIcon from "@lucide/svelte/icons/users-round";
  import CrownIcon from "@lucide/svelte/icons/crown";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import { partySession } from "$lib/party/session.svelte";
  import { session } from "$lib/state/session.svelte";
  import { href, router } from "$lib/state/router.svelte";
  import { errorMessage } from "$lib/api/client";

  let { id, embedded = false, onback, onbrowse }: { id: string; embedded?: boolean; onback?: () => void; onbrowse?: () => void } = $props();
  let failure = $state<string | null>(null);
  let confirmLeave = $state(false);
  const party = $derived(partySession.connection?.id === id ? partySession.connection.party : null);
  const isHost = $derived(party?.host_user_id === session.user?.id);
  onMount(() => {
    void partySession.join(id).catch((error) => (failure = errorMessage(error)));
  });

  function leave() {
    partySession.leave();
    if (onback) onback(); else router.go(href.parties());
  }
</script>

{#snippet content()}
  {#if embedded}<h3 class="text-lg font-semibold">{party?.name ?? "Watch party"}</h3>
  {:else}<PageHeader title={party?.name ?? "Watch party"} description="Browse independently. When anyone starts a movie or episode, everyone opens it together. Playback is shared; volume and subtitles are yours." />{/if}
  {#if failure}
    <p role="alert" class="text-sm text-destructive">{failure}</p>
    {#if onback}<Button onclick={onback} variant="outline">Back to parties</Button>
    {:else}<Button href={href.parties()} variant="outline">Back to parties</Button>{/if}
  {:else if party}
    <section class="flex flex-col gap-5 rounded-xl border bg-card p-5" aria-label="Party members">
      <h2 class="flex items-center gap-2 font-medium"><UsersRoundIcon class="size-4" />Members · {party.members.length}</h2>
      <ul class="flex flex-col gap-3">
        {#each party.members as member (member.user_id)}
          <li class="flex items-center gap-2 text-sm">
            <span>{member.username}</span>
            {#if member.user_id === party.host_user_id}<CrownIcon class="size-3.5 text-muted-foreground" aria-label="Creator" />{/if}
            {#if member.user_id === session.user?.id}<span class="text-xs text-muted-foreground">You</span>{/if}
          </li>
        {/each}
      </ul>
      <div class="flex flex-wrap gap-2">
        {#if onbrowse}<Button onclick={onbrowse}>Browse library</Button>
        {:else}<Button href={href.home()}>Browse library</Button>{/if}
        {#if party.item_id}<Button variant="outline" href={href.watch(party.item_id)}>Return to playback</Button>{/if}
        <Button variant="ghost" onclick={() => (confirmLeave = !confirmLeave)}>{isHost ? "End party" : "Leave party"}</Button>
      </div>
      {#if confirmLeave}
        <div class="flex flex-wrap items-center gap-3 text-sm" role="status">
          <p>{isHost ? "End this party for everyone?" : "Leave this party? You can join again later."}</p>
          <Button size="sm" variant="outline" onclick={() => (confirmLeave = false)}>Stay</Button>
          <Button size="sm" variant="destructive" onclick={leave}>{isHost ? "End party" : "Leave"}</Button>
        </div>
      {/if}
    </section>
  {:else}
    <div class="flex items-center gap-2 text-sm text-muted-foreground" role="status"><Spinner />Joining party…</div>
  {/if}
{/snippet}
{#if embedded}<div class="flex flex-col gap-5">{@render content()}</div>
{:else}<Page class="max-w-3xl">{@render content()}</Page>{/if}
