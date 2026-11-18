<script lang="ts">
  import HeartIcon from "@lucide/svelte/icons/heart";
  import { Button, type ButtonSize } from "$lib/components/ui/button";
  import { itemsApi } from "$lib/api/endpoints";
  import type { Item } from "$lib/api/types";
  import { catalog } from "$lib/state/catalog.svelte";

  // `size` matches the surrounding action row (hero actions are `lg`).
  let { items, label = "Favorite", size = "lg", iconOnly = false }: { items: Item[]; label?: string; size?: ButtonSize; iconOnly?: boolean } = $props();
  let busy = $state(false);
  let error = $state(false);
  const active = $derived(items.length > 0 && items.every((item) => item.favorite));

  async function toggle() {
    const next = !active;
    busy = true;
    error = false;
    try {
      for (const item of items) {
        await itemsApi.favorite(item.id, next);
        catalog.patch(item.id, { favorite: next });
      }
    } catch { error = true; }
    finally { busy = false; }
  }
</script>

<Button variant={iconOnly ? "ghost" : "outline"} {size} class="cursor-pointer" disabled={busy || !items.length} aria-pressed={active} aria-label={iconOnly ? label : undefined} title={iconOnly ? label : undefined} onclick={toggle}>
  <HeartIcon data-icon="inline-start" class={active ? "fill-current" : ""} />
  {#if !iconOnly}{label}{/if}
</Button>
{#if error}<span role="status" class="text-xs text-destructive">Could not save favorites. Try again.</span>{/if}
