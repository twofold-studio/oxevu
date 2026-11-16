<script lang="ts">
  import { session } from "$lib/state/session.svelte";
  import { href } from "$lib/state/router.svelte";
  import { settingsNavFor, type SettingsPageId } from "$lib/state/settings-pages";
  import { cn } from "$lib/utils";
  import { pageIcons } from "./page-icons";

  // Grouped settings destinations. `rail` is the desktop aside; `list` is the
  // phone index with full-width touch rows.
  let { active = null, variant }: { active?: SettingsPageId | null; variant: "rail" | "list" } = $props();

  const groups = $derived(settingsNavFor(session.user));
</script>

<div class={cn("flex flex-col", variant === "rail" ? "gap-5" : "gap-6")}>
  {#each groups as group (group.id)}
    <section class="flex flex-col gap-1" aria-labelledby={`settings-group-${variant}-${group.id}`}>
      <h2
        id={`settings-group-${variant}-${group.id}`}
        class="px-2.5 text-xs font-medium tracking-wider text-muted-foreground uppercase"
      >
        {group.label}
      </h2>
      <ul class={cn("flex flex-col", variant === "list" ? "divide-y rounded-xl border bg-card" : "gap-0.5")}>
        {#each group.pages as page (page.id)}
          {@const Icon = pageIcons[page.id]}
          {@const current = active === page.id}
          <li>
            <a
              href={href.settings(page.id)}
              aria-current={current ? "page" : undefined}
              class={cn(
                "relative flex items-center gap-2.5 text-sm transition-colors duration-200 focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:outline-none",
                variant === "rail"
                  ? "h-8 rounded-lg px-2.5 text-muted-foreground hover:bg-sidebar-accent hover:text-foreground"
                  : "h-12 px-4 hover:bg-accent",
                current &&
                  "bg-sidebar-accent font-medium text-foreground before:absolute before:inset-y-1.5 before:left-0 before:w-0.5 before:rounded-full before:bg-foreground",
              )}
            >
              <Icon class="size-4 shrink-0" />
              <span class="truncate">{page.label}</span>
            </a>
          </li>
        {/each}
      </ul>
    </section>
  {/each}
</div>
