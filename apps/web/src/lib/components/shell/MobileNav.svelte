<script lang="ts">
  import { router } from "$lib/state/router.svelte";
  import { cn } from "$lib/utils";
  import { browseNav, manageNav } from "./nav";

  const items = [...browseNav, ...manageNav];
</script>

<!-- Touch-sized bottom navigation replaces the rail below md. -->
<nav
  aria-label="Primary"
  class="fixed inset-x-0 bottom-0 z-40 grid grid-cols-4 border-t bg-background/95 pb-[env(safe-area-inset-bottom)] backdrop-blur md:hidden"
>
  {#each items as item (item.href)}
    {@const active = item.active(router.route)}
    <a
      href={item.href}
      aria-current={active ? "page" : undefined}
      class={cn(
        "flex h-14 flex-col items-center justify-center gap-1 text-[0.7rem] font-medium text-muted-foreground transition-colors focus-visible:bg-accent focus-visible:outline-none",
        active && "text-foreground",
      )}
    >
      <item.icon class="size-5" />
      {item.label}
    </a>
  {/each}
</nav>
