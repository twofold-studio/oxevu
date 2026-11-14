<script lang="ts">
  import { cn } from "$lib/utils";

  let { class: className }: { class?: string } = $props();
</script>

<!-- Same geometry as assets/logo.svg, drawn with the brand token. -->
<svg viewBox="0 0 128 128" aria-hidden="true" class={cn("size-6 shrink-0 text-brand", className)}>
  <path
    fill="currentColor"
    fill-rule="evenodd"
    d="M64 8a56 56 0 1 0 0 112A56 56 0 0 0 64 8Zm-11 32v48l38-24-38-24Z"
  />
</svg>
