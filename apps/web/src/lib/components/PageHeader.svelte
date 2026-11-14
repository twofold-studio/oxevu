<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    description,
    actions,
  }: { title: string; description?: string; actions?: Snippet } = $props();
</script>

<div class="mb-6 flex flex-wrap items-end justify-between gap-4">
  <div class="flex min-w-0 flex-col gap-1">
    <h1 class="text-xl font-semibold tracking-tight text-balance">{title}</h1>
    {#if description}
      <p class="max-w-2xl text-sm leading-6 text-muted-foreground">{description}</p>
    {/if}
  </div>
  {#if actions}
    <div class="flex shrink-0 items-center gap-2">{@render actions()}</div>
  {/if}
</div>
