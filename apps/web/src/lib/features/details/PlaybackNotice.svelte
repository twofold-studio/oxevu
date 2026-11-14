<script lang="ts">
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";
  import InfoIcon from "@lucide/svelte/icons/info";
  import CircleCheckIcon from "@lucide/svelte/icons/circle-check";
  import * as Alert from "$lib/components/ui/alert";
  import type { Item } from "$lib/api/types";
  import { checkCompatibility } from "$lib/media/compatibility";

  let { item }: { item: Item } = $props();

  const result = $derived(checkCompatibility(item));
</script>

{#if result.verdict === "unlikely"}
  <Alert.Root variant="destructive">
    <TriangleAlertIcon />
    <Alert.Title>This browser probably can't play this file</Alert.Title>
    <Alert.Description>
      It reports no support for {result.blockers.join(" or ")}. Oxevu plays the original file and can't convert
      video yet. Try another browser, or keep using your current player for this title.
    </Alert.Description>
  </Alert.Root>
{:else if result.verdict === "likely"}
  <p class="flex items-center gap-2 text-sm text-muted-foreground">
    <CircleCheckIcon class="size-4" />
    This browser reports support for this file's format.
  </p>
{:else}
  <p class="flex items-start gap-2 text-sm text-muted-foreground">
    <InfoIcon class="mt-0.5 size-4 shrink-0" />
    Plays the original file without conversion. Support for this format varies by browser; if it fails, the
    player will say why.
  </p>
{/if}
