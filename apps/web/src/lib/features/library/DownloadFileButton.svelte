<script lang="ts">
  import DownloadIcon from "@lucide/svelte/icons/download";
  import { itemsApi } from "$lib/api/endpoints";
  import type { Item } from "$lib/api/types";
  import { Button } from "$lib/components/ui/button";
  import { fileAvailable } from "$lib/state/catalog.svelte";
  import { session } from "$lib/state/session.svelte";

  let { item, compact = false }: { item: Item; compact?: boolean } = $props();
</script>

{#if session.user?.can_play && fileAvailable(item)}
  <Button
    href={itemsApi.downloadUrl(item.id)}
    download=""
    variant={compact ? "ghost" : "outline"}
    size={compact ? "icon" : "lg"}
    class="relative z-10 cursor-pointer"
    aria-label={`Download file for ${item.title}`}
    title="Download file"
  >
    <DownloadIcon data-icon="inline-start" />
    {#if !compact}Download file{/if}
  </Button>
{/if}
