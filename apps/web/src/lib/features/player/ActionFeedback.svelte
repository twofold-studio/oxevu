<script lang="ts">
  import RewindIcon from "@lucide/svelte/icons/rewind";
  import FastForwardIcon from "@lucide/svelte/icons/fast-forward";
  import Volume2Icon from "@lucide/svelte/icons/volume-2";
  import VolumeXIcon from "@lucide/svelte/icons/volume-x";
  import PlayIcon from "@lucide/svelte/icons/play";
  import PauseIcon from "@lucide/svelte/icons/pause";
  let { action }: { action: { id: number; kind: "back" | "forward" | "volume" | "mute" | "play" | "pause"; label: string } | null } = $props();
</script>

{#if action}
  <div class="pointer-events-none absolute inset-0 overflow-hidden" role="status" aria-live="polite" aria-atomic="true">
    {#key action.id}
    <div class="feedback absolute -translate-x-1/2 -translate-y-1/2"
      style:left={action.kind === "back" ? "24%" : action.kind === "forward" ? "76%" : "50%"}
      style:top="50%"
      style:--travel={action.kind === "back" ? "-8px" : action.kind === "forward" ? "8px" : "0px"}>
    <div class="pulse relative flex min-w-28 flex-col items-center gap-2 rounded-full bg-black/35 px-4 py-5 text-white/85 backdrop-blur-sm sm:min-w-32 sm:px-6">
      <span class="ripple absolute inset-0 rounded-full border border-white/20" aria-hidden="true"></span>
      <span class="symbol" aria-hidden="true">
      {#if action.kind === "back"}<RewindIcon class="size-8" />
      {:else if action.kind === "forward"}<FastForwardIcon class="size-8" />
      {:else if action.kind === "mute"}<VolumeXIcon class="size-8" />
      {:else if action.kind === "play"}<PlayIcon class="size-8 fill-current" />
      {:else if action.kind === "pause"}<PauseIcon class="size-8 fill-current" />
      {:else}<Volume2Icon class="size-8" />{/if}
      </span>
      <span class="text-sm font-medium tabular-nums">{action.label}</span>
    </div>
    </div>
    {/key}
  </div>
{/if}

<style>
  .pulse { animation: feedback-pulse 850ms cubic-bezier(.22, 1, .36, 1) both; }
  .ripple { animation: feedback-ripple 700ms cubic-bezier(.16, 1, .3, 1) both; }
  .symbol { animation: feedback-direction 450ms cubic-bezier(.22, 1, .36, 1) both; }
  @keyframes feedback-pulse {
    0% { opacity: 0; transform: scale(0.92); filter: blur(3px); }
    22%, 60% { opacity: 1; transform: scale(1); filter: blur(0); }
    100% { opacity: 0; transform: scale(1.04); filter: blur(1px); }
  }
  @keyframes feedback-ripple {
    from { opacity: 0.5; transform: scale(0.94); }
    to { opacity: 0; transform: scale(1.2); }
  }
  @keyframes feedback-direction {
    from { transform: translateX(calc(var(--travel) * -1)); }
    to { transform: translateX(var(--travel)); }
  }
  @media (prefers-reduced-motion: reduce) {
    .pulse, .ripple, .symbol { animation: none; }
    .ripple { display: none; }
  }
</style>
