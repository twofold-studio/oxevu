<script lang="ts">
  import RewindIcon from "@lucide/svelte/icons/rewind";
  import FastForwardIcon from "@lucide/svelte/icons/fast-forward";
  import Volume2Icon from "@lucide/svelte/icons/volume-2";
  import VolumeXIcon from "@lucide/svelte/icons/volume-x";
  let { action }: { action: { kind: "back" | "forward" | "volume" | "mute"; label: string } | null } = $props();
</script>

{#if action}
  <div class="pointer-events-none absolute inset-0 grid place-items-center" role="status" aria-live="polite" aria-atomic="true">
    <div class="animate-fade flex min-w-32 flex-col items-center gap-2 rounded-2xl bg-black/45 px-6 py-5 text-white/85 backdrop-blur-sm">
      {#if action.kind === "back"}<RewindIcon class="size-8" />
      {:else if action.kind === "forward"}<FastForwardIcon class="size-8" />
      {:else if action.kind === "mute"}<VolumeXIcon class="size-8" />
      {:else}<Volume2Icon class="size-8" />{/if}
      <span class="text-sm font-medium tabular-nums">{action.label}</span>
    </div>
  </div>
{/if}
