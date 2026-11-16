<script lang="ts">
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";
  import * as Alert from "$lib/components/ui/alert";
  import type { Item } from "$lib/api/types";
  import { checkCompatibility } from "$lib/media/compatibility";
  import { session } from "$lib/state/session.svelte";

  // A pre-play hint only; the server decides delivery when playback starts
  // and the player shows what it actually used.
  let { item }: { item: Item } = $props();

  const result = $derived(checkCompatibility(item));
  const canConvert = $derived(session.user?.can_transcode ?? false);
</script>

{#if result.verdict === "unlikely" && !canConvert}
  <Alert.Root variant="destructive">
    <TriangleAlertIcon />
    <Alert.Title>This browser probably can't play this file</Alert.Title>
    <Alert.Description>
      It reports no support for {result.blockers.join(" or ")}, and your account can't use conversion. Ask an
      administrator, or try another browser.
    </Alert.Description>
  </Alert.Root>
{/if}
