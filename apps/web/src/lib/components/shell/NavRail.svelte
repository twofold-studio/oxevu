<script lang="ts">
  import * as Tooltip from "$lib/components/ui/tooltip";
  import { router } from "$lib/state/router.svelte";
  import { session } from "$lib/state/session.svelte";
  import { cn } from "$lib/utils";
  import { browseNav, type NavItem } from "./nav";
</script>

{#snippet link(item: NavItem)}
  {@const active = item.active(router.route)}
  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <a
          {...props}
          href={item.href}
          aria-label={item.label}
          aria-current={active ? "page" : undefined}
          class={cn(
            "relative flex size-10 items-center justify-center rounded-lg text-muted-foreground transition-colors duration-200 hover:bg-sidebar-accent hover:text-foreground focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:outline-none",
            active &&
              "bg-sidebar-accent text-foreground before:absolute before:inset-y-2.5 before:left-0 before:w-0.5 before:rounded-full before:bg-foreground",
          )}
        >
          <item.icon class="size-[1.1rem]" />
        </a>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content side="right" sideOffset={8}>{item.label}</Tooltip.Content>
  </Tooltip.Root>
{/snippet}

<nav
  aria-label="Primary"
  class="sticky top-16 hidden h-[calc(100dvh-4rem)] w-[3.5rem] shrink-0 flex-col items-center gap-1 border-r bg-sidebar py-3 md:flex"
>
  {#each browseNav(session.user) as item (item.href)}
    {@render link(item)}
  {/each}
</nav>
