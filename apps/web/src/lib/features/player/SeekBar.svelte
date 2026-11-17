<script lang="ts">
  import { formatClock } from "$lib/media/format";
  import { itemsApi } from "$lib/api/endpoints";
  import { mergeReadyRanges } from "$lib/playback/ready-ranges";

  // Scrubbing updates a local value and seeks once on release, so dragging
  // doesn't fire a byte-range request for every pixel.
  let {
    current,
    duration,
    bufferedEnd,
    readyRanges = [],
    onseek,
    disabled = false,
    previewItemId,
  }: {
    current: number;
    duration: number;
    bufferedEnd: number;
    readyRanges?: { start: number; end: number }[];
    onseek: (time: number) => void;
    disabled?: boolean;
    previewItemId?: string;
  } = $props();

  let scrub = $state<number | null>(null);
  let hover = $state<number | null>(null);
  let frame = $state<{ id: string; second: number } | null>(null);
  let loaded = $state(false);
  const shown = $derived(scrub ?? current);
  const previewTime = $derived(disabled ? null : scrub ?? hover);
  const previewSecond = $derived(previewTime === null ? null : Math.floor(Math.min(previewTime, Math.max(0, duration - 0.1)) / 5) * 5);
  const merged = $derived(mergeReadyRanges(readyRanges, duration));
  const pct = (value: number) => (duration > 0 ? Math.min(Math.max(value / duration, 0), 1) * 100 : 0);

  $effect(() => {
    const second = previewSecond;
    const id = previewItemId;
    frame = null;
    loaded = false;
    if (!id || second === null) return;
    const timer = setTimeout(() => { frame = { id, second }; }, 150);
    return () => clearTimeout(timer);
  });

  function pointerTime(event: PointerEvent & { currentTarget: HTMLInputElement }) {
    if (event.pointerType === "touch" || disabled || !duration) return;
    const { left, width } = event.currentTarget.getBoundingClientRect();
    if (width > 0) hover = Math.min(1, Math.max(0, (event.clientX - left) / width)) * duration;
  }
</script>

<div class="group/seek relative flex h-5 items-center">
  <div class="pointer-events-none absolute inset-x-0 h-1 overflow-hidden rounded-full bg-white/20 transition-[height] group-hover/seek:h-1.5">
    {#each merged as range (range)}
      <div class="absolute inset-y-0 bg-white/40" style:left="{pct(range.start)}%" style:width="{Math.max(0, pct(range.end) - pct(range.start))}%"></div>
    {/each}
    <div class="absolute inset-y-0 left-0 bg-brand" style:width="{pct(shown)}%"></div>
  </div>
  <input
    type="range"
    min="0"
    max={duration || 0}
    step="0.1"
    value={shown}
    disabled={disabled || !duration}
    aria-label="Seek"
    aria-valuetext={`${formatClock(shown)} of ${formatClock(duration)}`}
    class="seek relative h-5 w-full cursor-pointer appearance-none bg-transparent disabled:cursor-default"
    onpointermove={pointerTime}
    onpointerleave={() => hover = null}
    onpointercancel={() => { hover = null; scrub = null; }}
    oninput={(event) => (scrub = event.currentTarget.valueAsNumber)}
    onchange={(event) => {
      onseek(event.currentTarget.valueAsNumber);
      scrub = null;
    }}
  />
  {#if previewTime !== null}
    <div
      class="pointer-events-none absolute bottom-6 z-10 flex w-40 -translate-x-1/2 flex-col items-center gap-1"
      style:left={`clamp(5rem, ${pct(previewTime)}%, calc(100% - 5rem))`}
      aria-hidden="true"
    >
      {#if frame}
        {#key `${frame.id}:${frame.second}`}
          <img
            src={itemsApi.seekPreviewUrl(frame.id, frame.second)}
            alt=""
            class="aspect-video w-full rounded-md border border-white/20 bg-black object-contain"
            class:invisible={!loaded}
            onload={() => loaded = true}
            onerror={() => frame = null}
          />
        {/key}
      {/if}
      <span class="rounded bg-black/90 px-1.5 py-0.5 font-mono text-xs tabular-nums">{formatClock(previewTime)}</span>
    </div>
  {/if}
</div>

<style>
  .seek::-webkit-slider-thumb {
    appearance: none;
    width: 0.8rem;
    height: 0.8rem;
    border-radius: 9999px;
    background: var(--foreground);
    opacity: 0;
    transition: opacity 180ms ease;
  }
  .seek::-moz-range-thumb {
    width: 0.8rem;
    height: 0.8rem;
    border: 0;
    border-radius: 9999px;
    background: var(--foreground);
    opacity: 0;
    transition: opacity 180ms ease;
  }
  .seek:hover::-webkit-slider-thumb,
  .seek:focus-visible::-webkit-slider-thumb {
    opacity: 1;
  }
  .seek:hover::-moz-range-thumb,
  .seek:focus-visible::-moz-range-thumb {
    opacity: 1;
  }
  .seek:focus-visible {
    outline: none;
  }
  .seek:focus-visible::-webkit-slider-thumb {
    box-shadow: 0 0 0 3px var(--background), 0 0 0 5px var(--ring);
  }
  .seek:focus-visible::-moz-range-thumb {
    box-shadow: 0 0 0 3px var(--background), 0 0 0 5px var(--ring);
  }
</style>
