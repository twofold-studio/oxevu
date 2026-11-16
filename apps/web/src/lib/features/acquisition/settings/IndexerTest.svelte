<script lang="ts">
  import RadioTowerIcon from "@lucide/svelte/icons/radio-tower";
  import CheckIcon from "@lucide/svelte/icons/check";
  import CircleAlertIcon from "@lucide/svelte/icons/circle-alert";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { errorMessage } from "$lib/api/client";
  import { enter, leave } from "$lib/motion";

  let { id, name }: { id: string; name: string } = $props();
  let phase = $state<"idle" | "testing" | "success" | "error">("idle");
  let failure = $state("");

  async function test() {
    if (phase === "testing") return;
    phase = "testing";
    failure = "";
    const began = performance.now();
    let result: "success" | "error" = "success";
    try {
      await acquisitionApi.testIndexer(id);
    } catch (error) {
      failure = errorMessage(error);
      result = "error";
    }
    // Keep fast responses from flashing an unfinished spinner transition.
    await new Promise((resolve) => setTimeout(resolve, Math.max(0, 450 - (performance.now() - began))));
    phase = result;
  }
</script>

<div class="flex flex-col items-end gap-1.5">
  <Button size="sm" variant="outline" class="w-28" disabled={phase === "testing"} aria-label={`Test ${name}`} aria-busy={phase === "testing"} onclick={test}>
    <span class="grid size-4 shrink-0 place-items-center" aria-hidden="true">
      {#key phase}
        <span class="col-start-1 row-start-1 inline-flex" in:enter={{ delay: 0, y: 2 }} out:leave>
          {#if phase === "testing"}<Spinner />{:else if phase === "success"}<CheckIcon />{:else if phase === "error"}<CircleAlertIcon />{:else}<RadioTowerIcon />{/if}
        </span>
      {/key}
    </span>
    <span class="grid flex-1 place-items-center">
      {#key phase}<span class="col-start-1 row-start-1" in:enter={{ delay: 0, y: 0 }} out:leave>{phase === "testing" ? "Testing…" : phase === "success" ? "Test again" : "Test"}</span>{/key}
    </span>
  </Button>
  <div class="min-h-4 max-w-64 text-right text-xs" role="status" aria-live="polite">
    {#if phase === "success"}
      <span class="inline-flex items-center gap-1 text-muted-foreground" in:enter={{ delay: 0 }}><CheckIcon class="size-3" />Connection verified.</span>
    {:else if phase === "error"}
      <span class="text-destructive" in:enter={{ delay: 0 }}>{failure}</span>
    {/if}
  </div>
</div>
