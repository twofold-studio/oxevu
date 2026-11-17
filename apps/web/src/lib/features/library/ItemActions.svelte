<script lang="ts">
  import CheckIcon from "@lucide/svelte/icons/check";
  import HeartIcon from "@lucide/svelte/icons/heart";
  import { Button } from "$lib/components/ui/button";
  import { itemsApi } from "$lib/api/endpoints";
  import { catalog } from "$lib/state/catalog.svelte";
  import type { Item } from "$lib/api/types";
  let { item }: { item: Item } = $props();
  let busy = $state(false);
  let error = $state(false);
  async function change(favorite: boolean) {
    busy = true;
    error = false;
    try {
      if (favorite) {
        await itemsApi.favorite(item.id, !item.favorite);
        catalog.patch(item.id, { favorite: !item.favorite });
      } else {
        await itemsApi.saveProgress(item.id, { position_seconds: 0, completed: !item.completed });
        catalog.patch(item.id, { position_seconds: 0, completed: !item.completed });
      }
    } catch { error = true; }
    finally { busy = false; }
  }
</script>

<div class="relative z-10 flex shrink-0 items-center">
  <Button variant="ghost" size="icon" class="cursor-pointer" disabled={busy} aria-label="Watched" aria-pressed={item.completed} onclick={() => change(false)}><CheckIcon class={item.completed ? "text-brand" : "text-muted-foreground"} /></Button>
  <Button variant="ghost" size="icon" class="cursor-pointer" disabled={busy} aria-label="Favorite" aria-pressed={!!item.favorite} onclick={() => change(true)}><HeartIcon class={item.favorite ? "fill-current text-brand" : "text-muted-foreground"} /></Button>
  {#if error}<span role="status" class="text-xs text-destructive">Could not save</span>{/if}
</div>
