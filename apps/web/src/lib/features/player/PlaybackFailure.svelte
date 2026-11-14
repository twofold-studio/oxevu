<script lang="ts">
  import CircleAlertIcon from "@lucide/svelte/icons/circle-alert";
  import { Button } from "$lib/components/ui/button";

  let {
    title,
    detail,
    backHref,
    backLabel = "Back",
    onretry,
  }: { title: string; detail: string; backHref: string; backLabel?: string; onretry?: () => void } =
    $props();
</script>

<div class="absolute inset-0 z-10 grid place-items-center bg-black/85 p-6" role="alert">
  <div class="flex max-w-md flex-col items-center gap-4 text-center">
    <CircleAlertIcon class="size-8 text-white/70" />
    <div class="flex flex-col gap-2">
      <h2 class="text-lg font-semibold tracking-tight">{title}</h2>
      <p class="text-sm leading-6 text-white/70">{detail}</p>
    </div>
    <div class="flex gap-2">
      <Button href={backHref} variant="outline">{backLabel}</Button>
      {#if onretry}<Button onclick={onretry}>Try again</Button>{/if}
    </div>
  </div>
</div>
