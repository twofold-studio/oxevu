<script lang="ts">
  import type { Snippet } from "svelte";

  // One framed group with internal hairlines rather than detached cards.
  // The inner grid overhangs by 1px so the outer frame hides edge borders.
  let { children, label }: { children: Snippet; label: string } = $props();
</script>

<div class="overflow-hidden rounded-xl border">
  <ul
    aria-label={label}
    class="-mr-px -mb-px grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 2xl:grid-cols-5"
  >
    {@render children()}
  </ul>
</div>
