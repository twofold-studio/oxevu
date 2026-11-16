<script lang="ts">
  import { onMount } from "svelte";
  import { session } from "$lib/state/session.svelte";
  import { partySession } from "$lib/party/session.svelte";

  onMount(() => {
    if (session.user) partySession.restore(session.user.id);
    return () => partySession.stop();
  });
  $effect(() => {
    const state = partySession.connection?.state;
    if (state === "closed" || state === "failed") partySession.checkClosed();
  });
</script>
