<script lang="ts">
  import { router } from "$lib/state/router.svelte";
  import { session } from "$lib/state/session.svelte";
  import { cn } from "$lib/utils";
  import { browseNav, manageNav } from "./nav";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";

  const items = $derived([...browseNav(session.user), ...manageNav]);
  const primary = $derived(items.filter((item) => ["Home", "Discover", "Requests", "Activity"].includes(item.label)));
  const more = $derived(items.filter((item) => !primary.includes(item)));
  const moreActive = $derived(more.some((item) => item.active(router.route)));
</script>

<!-- Touch-sized bottom navigation replaces the rail below md. -->
<nav
  aria-label="Primary"
  class="fixed inset-x-0 bottom-0 z-40 grid auto-cols-fr grid-flow-col border-t bg-background/95 pb-[env(safe-area-inset-bottom)] backdrop-blur md:hidden"
>
  {#each primary as item (item.href)}
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
  <DropdownMenu.Root>
    <DropdownMenu.Trigger class={cn("flex h-14 flex-col items-center justify-center gap-1 text-[0.7rem] font-medium text-muted-foreground transition-colors focus-visible:bg-accent focus-visible:outline-none", moreActive && "text-foreground")}>
      <EllipsisIcon class="size-5" />More
    </DropdownMenu.Trigger>
    <DropdownMenu.Content side="top" align="end">
      <DropdownMenu.Group>
        {#each more as item (item.href)}
          <DropdownMenu.Item onclick={() => window.location.hash = item.href}><item.icon />{item.label}</DropdownMenu.Item>
        {/each}
      </DropdownMenu.Group>
    </DropdownMenu.Content>
  </DropdownMenu.Root>
</nav>
