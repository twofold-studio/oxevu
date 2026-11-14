<script lang="ts">
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";
  import InfoIcon from "@lucide/svelte/icons/info";
  import CircleCheckIcon from "@lucide/svelte/icons/circle-check";
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
{:else if result.verdict === "unlikely"}
  <p class="flex items-start gap-2 text-sm text-muted-foreground">
    <InfoIcon class="mt-0.5 size-4 shrink-0" />
    This browser can't play {result.blockers.join(" or ")} directly, so the server will convert it if conversion
    is available.
  </p>
{:else if result.verdict === "likely"}
  <p class="flex items-center gap-2 text-sm text-muted-foreground">
    <CircleCheckIcon class="size-4" />
    Plays the original file in this browser.
  </p>
{:else}
  <p class="flex items-start gap-2 text-sm text-muted-foreground">
    <InfoIcon class="mt-0.5 size-4 shrink-0" />
    Tries the original file first.{canConvert ? " If this browser can't play it, the player switches to a converted stream." : ""}
  </p>
{/if}
