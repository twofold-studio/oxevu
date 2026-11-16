<script lang="ts">
  import type { Snippet } from "svelte";
  import { settingsPageLabel, type SettingsPageId } from "$lib/state/settings-pages";

  // Shared heading for every settings page: group eyebrow, page title, one
  // sentence about what the page controls, and optional page actions.
  let {
    page,
    description,
    actions,
    children,
  }: { page: SettingsPageId; description: string; actions?: Snippet; children: Snippet } = $props();

  const label = $derived(settingsPageLabel(page));
</script>

<div class="flex flex-col gap-6">
  <header class="flex flex-wrap items-end justify-between gap-4">
    <div class="flex min-w-0 flex-col gap-1">
      <p class="text-xs font-medium tracking-wider text-muted-foreground uppercase">{label.group}</p>
      <h1 class="text-xl font-semibold tracking-tight text-balance">{label.page}</h1>
      <p class="max-w-2xl text-sm leading-6 text-muted-foreground">{description}</p>
    </div>
    {#if actions}<div class="flex shrink-0 items-center gap-2">{@render actions()}</div>{/if}
  </header>
  {@render children()}
</div>
