<script lang="ts">
  import ThumbsUpIcon from "@lucide/svelte/icons/thumbs-up";
  import ThumbsDownIcon from "@lucide/svelte/icons/thumbs-down";
  import { Button } from "$lib/components/ui/button";
  import { itemsApi } from "$lib/api/endpoints";
  import { catalog } from "$lib/state/catalog.svelte";
  import type { Item } from "$lib/api/types";

  let { item }: { item: Item } = $props();
  let busy = $state(false);
  let opinion = $derived(item.reaction ?? null);
  let error = $state(false);

  async function toggle(value: "like" | "dislike") {
    busy = true;
    error = false;
    const next = opinion === value ? null : value;
    try {
      const { value: confirmed } = await itemsApi.reaction(item.id, next);
      opinion = confirmed;
      catalog.patch(item.id, { reaction: confirmed });
    } catch {
      error = true;
    } finally {
      busy = false;
    }
  }
</script>

<div class="flex flex-col gap-1.5">
  <span class="text-xs font-medium text-muted-foreground">Your opinion</span>
  <div class="flex items-center gap-1">
    <Button
      variant="ghost"
      size="icon"
      disabled={busy}
      aria-label="Like"
      aria-pressed={opinion === "like"}
      onclick={() => toggle("like")}
    >
      <ThumbsUpIcon
        class={opinion === "like" ? "fill-current text-brand" : "text-muted-foreground"}
      />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      disabled={busy}
      aria-label="Dislike"
      aria-pressed={opinion === "dislike"}
      onclick={() => toggle("dislike")}
    >
      <ThumbsDownIcon
        class={opinion === "dislike" ? "fill-current text-destructive" : "text-muted-foreground"}
      />
    </Button>
    {#if error}<span role="status" class="text-xs text-destructive">Could not save</span>{/if}
  </div>
</div>
