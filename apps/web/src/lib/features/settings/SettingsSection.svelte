<script lang="ts">
  import type { Component, Snippet } from "svelte";

  // Gitadel settings row: explanation left, controls right; stacked on phones.
  let {
    icon: Icon,
    title,
    description,
    children,
  }: { icon: Component; title: string; description: string; children: Snippet } = $props();
</script>

<section class="rounded-xl border bg-card">
  <div class="grid gap-5 p-5 md:grid-cols-[minmax(12rem,0.72fr)_minmax(0,1.5fr)] md:gap-10 md:p-6">
    <div class="flex gap-3">
      <Icon class="mt-1 size-4 shrink-0 text-muted-foreground" />
      <div class="flex flex-col gap-1">
        <h2 class="text-base leading-snug font-medium">{title}</h2>
        <p class="text-sm leading-6 text-muted-foreground">{description}</p>
      </div>
    </div>
    <div class="max-w-2xl min-w-0">{@render children()}</div>
  </div>
</section>
