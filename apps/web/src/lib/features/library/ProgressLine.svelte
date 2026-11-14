<script lang="ts">
  import { cn } from "$lib/utils";

  let { value, class: className }: { value: number; class?: string } = $props();

  const percent = $derived(Math.round(Math.min(Math.max(value, 0), 1) * 100));
</script>

<div
  role="progressbar"
  aria-label="Watched"
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuenow={percent}
  class={cn("h-0.5 w-full overflow-hidden bg-foreground/10", className)}
>
  <div class="h-full bg-brand" style:width="{percent}%"></div>
</div>
