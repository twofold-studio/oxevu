<script lang="ts">
  import type { Snippet } from "svelte";
  import { onMount } from "svelte";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import * as Alert from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import { catalog, catalogVisibleRefresh } from "$lib/state/catalog.svelte";

  // Loads the shared item list and renders loading/error states around a
  // page. Pages render their own skeleton so it matches their layout.
  let { children, skeleton }: { children: Snippet; skeleton: Snippet } = $props();

  onMount(() => {
    void catalog.ensure();
    return catalogVisibleRefresh();
  });
</script>

{#if catalog.status === "ready"}
  {@render children()}
{:else if catalog.status === "error"}
  <Alert.Root variant="destructive">
    <Alert.Title>Couldn't load your library</Alert.Title>
    <Alert.Description>{catalog.error}</Alert.Description>
    <Alert.Action>
      <Button variant="outline" size="sm" onclick={() => catalog.ensure(true)}>
        <RotateCwIcon data-icon="inline-start" />
        Retry
      </Button>
    </Alert.Action>
  </Alert.Root>
{:else}
  <div aria-busy="true" aria-label="Loading library">
    {@render skeleton()}
  </div>
{/if}
