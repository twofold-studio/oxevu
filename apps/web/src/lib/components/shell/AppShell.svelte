<script lang="ts">
  import type { Snippet } from "svelte";
  import AppHeader from "./AppHeader.svelte";
  import NavRail from "./NavRail.svelte";
  import MobileNav from "./MobileNav.svelte";

  let { children }: { children: Snippet } = $props();
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
    <NavRail />
    <main id="main" tabindex="-1" class="min-w-0 flex-1 pb-20 outline-none md:pb-0">
      {@render children()}
    </main>
  </div>
  <MobileNav />
</div>
