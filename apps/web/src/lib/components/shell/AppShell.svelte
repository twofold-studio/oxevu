<script lang="ts">
  import type { Snippet } from "svelte";
  import AppHeader from "./AppHeader.svelte";
  import NavRail from "./NavRail.svelte";
  import MobileNav from "./MobileNav.svelte";
  import SettingsNav from "$lib/features/settings/navigation/SettingsNav.svelte";
  import { router } from "$lib/state/router.svelte";
  import { cn } from "$lib/utils";

  let { children }: { children: Snippet } = $props();

  // Settings swap the application navigation for their own; leaving restores it.
  const inSettings = $derived(router.route.name === "settings");
</script>

<a
  href="#main"
  class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-50 focus:rounded-md focus:bg-primary focus:px-3 focus:py-2 focus:text-sm focus:text-primary-foreground"
  onclick={(event) => {
    event.preventDefault();
    document.getElementById("main")?.focus();
  }}>Skip to content</a
>

<div class="flex min-h-dvh flex-col">
  <AppHeader />
  <div class="flex flex-1">
    {#if inSettings}<SettingsNav />{:else}<NavRail />{/if}
    <main id="main" tabindex="-1" class={cn("min-w-0 flex-1 outline-none md:pb-0", !inSettings && "pb-20")}>
      {@render children()}
    </main>
  </div>
  {#if !inSettings}<MobileNav />{/if}
</div>
