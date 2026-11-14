<script lang="ts">
  import { onMount } from "svelte";
  import { Toaster } from "$lib/components/ui/sonner";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import { session } from "$lib/state/session.svelte";
  import { router } from "$lib/state/router.svelte";
  import AuthScreen from "$lib/features/auth/AuthScreen.svelte";
  import Unreachable from "$lib/features/auth/Unreachable.svelte";
  import AppShell from "$lib/components/shell/AppShell.svelte";
  import PlayerPage from "$lib/features/player/PlayerPage.svelte";
  import Routes from "./Routes.svelte";

  onMount(() => {
    void session.load();
  });
</script>

<Tooltip.Provider delayDuration={300}>
  {#if session.phase === "loading"}
    <div class="grid min-h-dvh place-items-center" aria-busy="true">
      <span class="sr-only">Loading Oxevu</span>
    </div>
  {:else if session.phase === "unreachable"}
    <Unreachable message={session.error} onretry={() => session.load()} />
  {:else if session.phase === "setup" || session.phase === "signed-out"}
    <AuthScreen mode={session.phase === "setup" ? "setup" : "sign-in"} notice={session.error} />
  {:else if router.route.name === "watch"}
    <!-- The player owns the whole viewport; the shell would only get in the way. -->
    {#key router.route.id}
      <PlayerPage id={router.route.id} fromStart={router.route.fromStart} />
    {/key}
  {:else}
    <AppShell>
      <Routes route={router.route} />
    </AppShell>
  {/if}
</Tooltip.Provider>

<Toaster position="bottom-right" />
