<script lang="ts">
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import { router } from "$lib/state/router.svelte";
  import { isSettingsPage } from "$lib/state/settings-pages";
  import SettingsNavList from "./SettingsNavList.svelte";

  // Replaces the application rail while settings are open (md and wider).
  // Phones use the settings index page instead; see SettingsPage.
  const active = $derived(
    router.route.name === "settings" && router.route.target && isSettingsPage(router.route.target)
      ? router.route.target
      : null,
  );
</script>

<nav
  aria-label="Settings"
  class="sticky top-16 hidden h-[calc(100dvh-4rem)] w-60 shrink-0 flex-col border-r bg-sidebar md:flex"
>
  <div class="flex flex-col gap-3 border-b px-3 py-3">
    <a
      href={router.returnTo}
      class="flex h-8 items-center gap-2 rounded-lg px-2.5 text-sm text-muted-foreground transition-colors duration-200 hover:bg-sidebar-accent hover:text-foreground focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:outline-none"
    >
      <ArrowLeftIcon class="size-4" />
      Back to library
    </a>
    <p class="px-2.5 text-base font-semibold tracking-tight">Settings</p>
  </div>
  <div class="min-h-0 flex-1 overflow-y-auto px-3 py-4">
    <SettingsNavList variant="rail" {active} />
  </div>
</nav>
