<script lang="ts">
  import HeartIcon from "@lucide/svelte/icons/heart";
  import { Button } from "$lib/components/ui/button";
  import { itemsApi } from "$lib/api/endpoints";
  import type { Item } from "$lib/api/types";
  import { catalog } from "$lib/state/catalog.svelte";

  let { items, label = "Favorite" }: { items: Item[]; label?: string } = $props();
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

<Button variant="outline" disabled={busy || !items.length} aria-pressed={active} onclick={toggle}>
  <HeartIcon class={active ? "fill-current" : ""} />
  {label}
</Button>
{#if error}<span role="status" class="text-xs text-destructive">Could not save favorites. Try again.</span>{/if}
