<script lang="ts">
  import PlayIcon from "@lucide/svelte/icons/play";
  import PauseIcon from "@lucide/svelte/icons/pause";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import Volume2Icon from "@lucide/svelte/icons/volume-2";
  import VolumeXIcon from "@lucide/svelte/icons/volume-x";
  import MaximizeIcon from "@lucide/svelte/icons/maximize";
  import MinimizeIcon from "@lucide/svelte/icons/minimize";
  import type { Snippet } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { formatClock } from "$lib/media/format";
  import SeekBar from "./SeekBar.svelte";

  let {
    paused,
    current,
    duration,
    bufferedEnd,
    muted,
    volume,
    fullscreen,
    locked = false,
    extra,
    ontoggle,
    onseek,
    onskip,
    onmute,
    onvolume,
    onfullscreen,
  }: {
    paused: boolean;
    current: number;
    duration: number;
    bufferedEnd: number;
    muted: boolean;
    volume: number;
    fullscreen: boolean;
    /** Watch-party guests follow the host: no play, pause or seek. */
    locked?: boolean;
    extra?: Snippet;
    ontoggle: () => void;
    onseek: (time: number) => void;
    onskip: (delta: number) => void;
    onmute: () => void;
    onvolume: (value: number) => void;
    onfullscreen: () => void;
  } = $props();

  const silent = $derived(muted || volume === 0);
</script>

<div class="flex flex-col gap-2">
  <SeekBar {current} {duration} {bufferedEnd} {onseek} disabled={locked} />
  <div class="flex items-center gap-1">
    {#if !locked}
      <Button variant="ghost" size="icon-lg" onclick={ontoggle} aria-label={paused ? "Play (k)" : "Pause (k)"}>
        {#if paused}<PlayIcon />{:else}<PauseIcon />{/if}
      </Button>
      <Button variant="ghost" size="icon-lg" onclick={() => onskip(-10)} aria-label="Back 10 seconds (←)">
        <RotateCcwIcon />
      </Button>
      <Button variant="ghost" size="icon-lg" onclick={() => onskip(10)} aria-label="Forward 10 seconds (→)">
        <RotateCwIcon />
      </Button>
    {/if}
    <div class="group/vol flex items-center">
      <Button variant="ghost" size="icon-lg" onclick={onmute} aria-label={silent ? "Unmute (m)" : "Mute (m)"}>
        {#if silent}<VolumeXIcon />{:else}<Volume2Icon />{/if}
      </Button>
      <input
        type="range"
        min="0"
        max="1"
        step="0.05"
        value={silent ? 0 : volume}
        aria-label="Volume"
        class="hidden h-1 w-20 cursor-pointer accent-foreground sm:block"
        oninput={(event) => onvolume(event.currentTarget.valueAsNumber)}
      />
    </div>
    <span class="ml-2 font-mono text-xs text-white/80 tabular-nums">
      {formatClock(current)} <span class="text-white/50">/ {formatClock(duration)}</span>
    </span>
    <span class="flex-1"></span>
    {@render extra?.()}
    <Button
      variant="ghost"
      size="icon-lg"
      onclick={onfullscreen}
      aria-label={fullscreen ? "Exit full screen (f)" : "Full screen (f)"}
    >
      {#if fullscreen}<MinimizeIcon />{:else}<MaximizeIcon />{/if}
    </Button>
  </div>
</div>
