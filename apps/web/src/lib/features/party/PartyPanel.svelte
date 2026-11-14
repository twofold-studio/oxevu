<script lang="ts">
  import CrownIcon from "@lucide/svelte/icons/crown";
  import XIcon from "@lucide/svelte/icons/x";
  import ArrowRightLeftIcon from "@lucide/svelte/icons/arrow-right-left";
  import LogOutIcon from "@lucide/svelte/icons/log-out";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import type { Party } from "$lib/api/playback-types";
  import { session } from "$lib/state/session.svelte";
  import ItemPicker from "./ItemPicker.svelte";

  // Rendered inside the player stage (not portaled) so it stays visible in
  // full screen.
  let {
    party,
    isHost,
    onclose,
    onleave,
    onchangeitem,
  }: {
    party: Party;
    isHost: boolean;
    onclose: () => void;
    onleave: () => void;
    onchangeitem: (itemId: string) => Promise<void>;
  } = $props();

  let picking = $state(false);
  let choice = $state<string | null>(null);
  let changing = $state(false);
  let confirmLeave = $state(false);

  async function change() {
    if (!choice || changing) return;
    changing = true;
    try {
      await onchangeitem(choice);
      picking = false;
      choice = null;
    } finally {
      changing = false;
    }
  }
</script>

<aside
  class="absolute top-20 right-4 bottom-28 z-20 flex w-[min(20rem,calc(100vw-2rem))] flex-col gap-4 overflow-y-auto rounded-xl border border-white/10 bg-neutral-950/90 p-4 text-sm text-white backdrop-blur"
  aria-label="Watch party"
>
  <div class="flex items-start justify-between gap-2">
    <div class="flex min-w-0 flex-col">
      <span class="truncate font-medium">{party.name}</span>
      <span class="text-xs text-white/60">{isHost ? "You're hosting" : "The host controls playback"}</span>
    </div>
    <Button variant="ghost" size="icon" onclick={onclose} aria-label="Close party panel"><XIcon /></Button>
  </div>

  <section class="flex flex-col gap-2" aria-label="Members">
    <h3 class="text-xs font-medium tracking-wider text-white/60 uppercase">Members · {party.members.length}</h3>
    <ul class="flex flex-col gap-1">
      {#each party.members as member (member.user_id)}
        <li class="flex items-center gap-2">
          <span class="grid size-6 place-items-center rounded-full bg-white/10 text-[0.65rem] font-semibold" aria-hidden="true">
            {member.username.slice(0, 1).toUpperCase()}
          </span>
          <span class="truncate">{member.username}</span>
          {#if member.user_id === session.user?.id}<span class="text-xs text-white/50">(you)</span>{/if}
          {#if member.user_id === party.host_user_id}
            <CrownIcon class="ml-auto size-3.5 text-white/70" aria-label="Host" />
          {/if}
        </li>
      {/each}
    </ul>
  </section>

  {#if isHost}
    <section class="flex flex-col gap-2">
      {#if picking}
        <ItemPicker bind:value={choice} />
        <div class="flex gap-2">
          <Button size="sm" variant="outline" onclick={() => (picking = false)}>Cancel</Button>
          <Button size="sm" onclick={change} disabled={!choice || changing}>
            {#if changing}<Spinner data-icon="inline-start" />{/if}
            Play for everyone
          </Button>
        </div>
      {:else}
        <Button size="sm" variant="outline" onclick={() => (picking = true)}>
          <ArrowRightLeftIcon data-icon="inline-start" />
          Change title
        </Button>
      {/if}
    </section>
  {/if}

  <section class="mt-auto flex flex-col gap-2">
    {#if confirmLeave}
      <p class="text-xs text-white/70">{isHost ? "This ends the party for everyone." : "You can rejoin from Watch parties."}</p>
      <div class="flex gap-2">
        <Button size="sm" variant="outline" onclick={() => (confirmLeave = false)}>Stay</Button>
        <Button size="sm" variant="destructive" onclick={onleave}>{isHost ? "End party" : "Leave"}</Button>
      </div>
    {:else}
      <Button size="sm" variant="ghost" onclick={() => (confirmLeave = true)}>
        <LogOutIcon data-icon="inline-start" />
        {isHost ? "End party" : "Leave party"}
      </Button>
    {/if}
  </section>
</aside>
