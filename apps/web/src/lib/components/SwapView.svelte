<script lang="ts" generics="K extends string | number">
  import type { Snippet } from "svelte";
  import { enter, leave } from "$lib/motion";
  import { cn } from "$lib/utils";

  // Swaps one view for another when `key` changes: the old view fades out
  // (inert at once) while the new one rises in, both stacked in one grid cell
  // so the region keeps the taller height during the swap instead of
  // collapsing or stacking two views. The keyed each block hands each view
  // the key it was created with, so an outgoing view keeps showing its own
  // content even though `key` has already moved on; derive the view from the
  // snippet argument, not from outer state. Rapid changes simply stack more
  // fading views; nothing is awaited.
  let { key, children, class: className }: { key: K; children: Snippet<[K]>; class?: string } = $props();
</script>

<div class={cn("grid", className)}>
  {#each [key] as current (current)}
    <div class="col-start-1 row-start-1 min-w-0" in:enter out:leave>
      {@render children(current)}
    </div>
  {/each}
</div>
