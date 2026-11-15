<script lang="ts">
  import SearchIcon from "@lucide/svelte/icons/search";
  import { Button } from "$lib/components/ui/button";
  import type { Item } from "$lib/api/types";
  import { session } from "$lib/state/session.svelte";
  import IdentifyDialog from "./IdentifyDialog.svelte";

  let { item, onsaved }: { item: Item; onsaved?: (item: Item) => void } = $props();
  let open = $state(false);
  let opening = $state(0);
</script>

{#if session.user?.is_admin}
  <Button variant="outline" size="lg" onclick={() => { opening++; open = true; }}>
    <SearchIcon data-icon="inline-start" />Identify
  </Button>
  {#key opening}
    {#if opening > 0}<IdentifyDialog bind:open {item} {onsaved} />{/if}
  {/key}
{/if}
