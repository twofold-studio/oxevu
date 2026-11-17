<script lang="ts">
  import { onMount } from "svelte";
  import "lenis/dist/lenis.css";
  import { Toaster } from "$lib/components/ui/sonner";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import { session } from "$lib/state/session.svelte";
  import { router } from "$lib/state/router.svelte";
  import AuthScreen from "$lib/features/auth/AuthScreen.svelte";
  import Unreachable from "$lib/features/auth/Unreachable.svelte";
  import AppShell from "$lib/components/shell/AppShell.svelte";
  import PlayerPage from "$lib/features/player/PlayerPage.svelte";
  import PartyRoom from "$lib/features/party/PartyRoom.svelte";
  import PartyCoordinator from "$lib/features/party/PartyCoordinator.svelte";
  import PartyPlayback from "$lib/features/party/PartyPlayback.svelte";
  import { partySession } from "$lib/party/session.svelte";
  import Routes from "./Routes.svelte";
  import ServerPicker from "$lib/mobile/ServerPicker.svelte";
  import { isMobileApp, loadMobileServer } from "$lib/mobile/connection";
  import { App as NativeApp } from "@capacitor/app";

  let mobileReady = $state(!isMobileApp);
  let mobileServer = $state<string | null>(null);
  let chooseServer = $state(false);

  onMount(() => { if (!isMobileApp) return router.mountSmoothScroll(); });

  onMount(() => {
    void (async () => {
      try {
        mobileServer = await loadMobileServer();
        chooseServer = isMobileApp && !mobileServer;
        if (!chooseServer) await session.load();
      } catch {
        chooseServer = isMobileApp;
      } finally { mobileReady = true; }
    })();
    if (!isMobileApp) return;
    const showPicker = () => { chooseServer = true; };
    window.addEventListener("oxevu:choose-server", showPicker);
    const listener = NativeApp.addListener("backButton", ({ canGoBack }) => {
      if (chooseServer && mobileServer) chooseServer = false;
      else if (document.fullscreenElement) void document.exitFullscreen();
      else if (canGoBack) window.history.back();
      else void NativeApp.minimizeApp();
    });
    return () => {
      window.removeEventListener("oxevu:choose-server", showPicker);
      void listener.then((handle) => handle.remove());
    };
  });
</script>

<Tooltip.Provider delayDuration={300}>
  {#if mobileReady && chooseServer}
    <ServerPicker current={mobileServer} oncancel={() => { chooseServer = false; }} />
  {:else if !mobileReady || session.phase === "loading"}
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
      {#if partySession.connection}
        <PartyPlayback id={router.route.id} fromStart={router.route.fromStart} />
      {:else}
        <PlayerPage id={router.route.id} fromStart={router.route.fromStart} />
      {/if}
    {/key}
  {:else if router.route.name === "party"}
    <AppShell>
      {#key router.route.id}<PartyRoom id={router.route.id} />{/key}
    </AppShell>
  {:else}
    <AppShell>
      <Routes route={router.route} />
    </AppShell>
  {/if}
  {#if mobileReady && !chooseServer && session.phase === "signed-in"}<PartyCoordinator />{/if}
  {#if isMobileApp && mobileReady && !chooseServer && session.phase !== "signed-in"}
    <button class="fixed right-4 top-4 rounded-md border bg-background px-3 py-2 text-sm" onclick={() => { chooseServer = true; }}>Change server</button>
  {/if}
</Tooltip.Provider>

<Toaster position="bottom-right" />
