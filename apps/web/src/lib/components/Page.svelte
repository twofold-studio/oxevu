<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  let { children, class: className }: { children: Snippet; class?: string } = $props();
</script>

<div class={cn("animate-rise mx-auto w-full max-w-7xl px-4 py-6 sm:px-5 lg:px-8 lg:py-8", className)}>
  {@render children()}
</div>
