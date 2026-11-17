<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { partySession } from "$lib/party/session.svelte";
  import PartiesPage from "./PartiesPage.svelte";
  import PartyRoom from "./PartyRoom.svelte";

  let { open = $bindable(false) }: { open: boolean } = $props();
  // The header remounts this dialog on each opening.
  let room = $state<string | null>(partySession.connection?.id ?? null);
</script>

<Dialog.Root bind:open>
  <Dialog.Content size="wide">
    <Dialog.Header>
      <Dialog.Title>Watch parties</Dialog.Title>
      <Dialog.Description>Join or manage a party while keeping your place in the library.</Dialog.Description>
    </Dialog.Header>
    {#if room}
      {#key room}<PartyRoom id={room} embedded onback={() => (room = null)} onbrowse={() => (open = false)} />{/key}
    {:else}
      <PartiesPage embedded onjoin={(id) => (room = id)} />
    {/if}
    <Dialog.Footer><Button variant="outline" onclick={() => (open = false)}>Close</Button></Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
