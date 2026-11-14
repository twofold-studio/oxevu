<script lang="ts">
  import type { Component, Snippet } from "svelte";

  let {
    id,
    icon: Icon,
    title,
    description,
    action,
  }: { id: string; icon: Component; title: string; description: string; action?: Snippet } = $props();
</script>

<div class="flex flex-wrap items-start justify-between gap-4">
  <div class="flex min-w-0 gap-3">
    <Icon class="mt-1 size-4 shrink-0 text-muted-foreground" />
    <div class="flex flex-col gap-1">
      <h2 {id} class="text-base leading-snug font-medium">{title}</h2>
      <p class="max-w-2xl text-sm leading-6 text-muted-foreground">{description}</p>
    </div>
  </div>
  {#if action}<div class="flex shrink-0 items-center gap-2">{@render action()}</div>{/if}
</div>
