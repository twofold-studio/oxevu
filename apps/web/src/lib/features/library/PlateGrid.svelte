<script lang="ts">
  import type { Snippet } from "svelte";

  // One framed group with internal hairlines rather than detached cards.
  // Columns stay few so posters read at roughly 220–320px, like a media
  // shelf rather than a thumbnail index.
  // The inner grid overhangs by 1px so the outer frame hides edge borders.
  let { children, label }: { children: Snippet; label: string } = $props();
</script>

<div class="overflow-hidden rounded-xl border">
  <ul
    aria-label={label}
    class="-mr-px -mb-px grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-6"
  >
    {@render children()}
  </ul>
</div>
