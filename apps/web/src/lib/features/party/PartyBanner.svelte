<script lang="ts">
  import UsersRoundIcon from "@lucide/svelte/icons/users-round";
  import PlayIcon from "@lucide/svelte/icons/play";
  import { Button } from "$lib/components/ui/button";
  import { partySession } from "$lib/party/session.svelte";
  import { href } from "$lib/state/router.svelte";
  import { plural } from "$lib/media/format";
  const party = $derived(partySession.connection?.party);
</script>

{#if party}
  <div class="flex min-w-0 items-center gap-3 border-b bg-muted/40 px-5 py-2 text-xs" aria-label="Current watch party">
    <UsersRoundIcon class="size-4 shrink-0 text-muted-foreground" />
    <a href={href.party(party.id)} class="min-w-0 flex-1 truncate py-1 font-medium hover:underline" aria-label={`Manage ${party.name}`}>{party.name}</a>
    <span class="shrink-0 text-muted-foreground">{plural(party.members.length, "member")}</span>
    <span class="hidden text-muted-foreground lg:inline">Anyone can start a title</span>
    {#if party.item_id}<Button size="icon-sm" variant="ghost" href={href.watch(party.item_id)} aria-label="Return to party playback" title="Return to playback"><PlayIcon /></Button>{/if}
  </div>
{/if}
