<script lang="ts">
  import type { Snippet } from "svelte";

  // Shared library shelves use the same spacing and density as Discover.
  let { children, label }: { children: Snippet; label: string } = $props();
</script>

<ul
  aria-label={label}
  class="grid grid-cols-2 gap-x-4 gap-y-6 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5"
>
  {@render children()}
</ul>
