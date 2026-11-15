<script lang="ts">
  import type { Component, Snippet } from "svelte";

  // One icon-led fact in a wrapping facts row. The icon is decorative; `label`
  // replaces the visible text for screen readers when the text alone is
  // ambiguous ("2019–2023" → "Series aired 2019 to 2023").
  let {
    icon: Icon,
    label,
    children,
  }: { icon?: Component<{ class?: string }>; label?: string; children: Snippet } = $props();
</script>

<!-- Facts wrap as whole items; text only breaks when one fact is wider than its row. -->
<li class="inline-flex min-w-0 items-center gap-1.5" title={label}>
  <!-- Lucide marks icons aria-hidden unless given an accessible name. -->
  {#if Icon}<Icon class="size-3.5 shrink-0 opacity-70" />{/if}
  {#if label}
    <span class="sr-only">{label}</span>
    <span aria-hidden="true">{@render children()}</span>
  {:else}
    {@render children()}
  {/if}
</li>
