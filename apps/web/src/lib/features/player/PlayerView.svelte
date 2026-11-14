<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import PlayIcon from "@lucide/svelte/icons/play";
  import CloudAlertIcon from "@lucide/svelte/icons/cloud-alert";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { catalog } from "$lib/state/catalog.svelte";
  import { episodeName, episodeNumberLabel } from "$lib/media/grouping";
  import { mediaOffset } from "$lib/playback/timeline";
  import type { PlaybackEngine } from "$lib/playback/engine.svelte";
  import { cn } from "$lib/utils";
  import PlayerControls from "./PlayerControls.svelte";
  import PlaybackFailure from "./PlaybackFailure.svelte";
  import DeliveryMenu from "./DeliveryMenu.svelte";
  import { ProgressReporter } from "./progress-reporter";
  import { createIdle } from "./idle.svelte";
  import { playerShortcut } from "./shortcuts";

  /** User-initiated changes, reported so a watch-party host can broadcast them. */
  export type PlayerIntent = { type: "play" | "pause" } | { type: "seek"; position: number };

  // The whole-viewport player. The engine owns sessions and absolute time;
  // this component renders it, saves progress and handles input. Parents
  // create one engine per item and key this component by item.
  let {
    engine,
    start,
    backHref,
    canTranscode,
    locked = false,
    autoplay = true,
    onintent,
    status,
    aside,
  }: {
    engine: PlaybackEngine;
    /** Absolute position to open at. */
    start: number;
    backHref: string;
    canTranscode: boolean;
    locked?: boolean;
    /** Start playing once the stream is ready (browsers may still block it). */
    autoplay?: boolean;
    onintent?: (intent: PlayerIntent) => void;
    /** Extra top-bar status, e.g. watch-party sync state. */
    status?: Snippet;
    /** Overlay content such as the party panel. */
    aside?: Snippet;
  } = $props();

  const item = $derived(engine.item);
  let stage = $state<HTMLDivElement | null>(null);
  let video = $state<HTMLVideoElement | null>(null);
  let mediaDuration = $state(Number.NaN);
  let paused = $state(true);
  let muted = $state(false);
  let volume = $state(1);
  let buffered = $state<{ start: number; end: number }[]>([]);
  let waiting = $state(false);
  let started = $state(false);
  let fullscreen = $state(false);
  let saveFailed = $state(false);

  const idle = createIdle(2500);
  const offset = $derived(engine.session ? mediaOffset(engine.session) : 0);
  const current = $derived(engine.absolute);
  // Item duration from the probe; otherwise what the stream reports.
  const duration = $derived(
    engine.duration ?? (Number.isFinite(mediaDuration) ? offset + mediaDuration : 0),
  );
  const bufferedEnd = $derived.by(() => {
    const local = engine.mediaTime;
    const range = buffered.find((r) => r.start <= local + 0.5 && r.end >= local);
    return range ? offset + range.end : 0;
  });
  const settled = $derived(engine.phase === "ready" && engine.pendingTarget === null);
  const controlsVisible = $derived(!idle.idle || paused || engine.problem !== null);
  const subtitle = $derived(
    item.kind === "episode"
      ? [item.show_title, episodeNumberLabel(item)].filter(Boolean).join(" · ")
      : null,
  );

  // Keyed by item in every parent, so capturing the initial item is intended.
  // svelte-ignore state_referenced_locally
  const reporter = new ProgressReporter(
    item.id,
    (progress) => {
      saveFailed = false;
      catalog.patch(item.id, progress);
    },
    () => (saveFailed = true),
  );

  onMount(() => {
    if (video) engine.attach(video);
    void engine.load(start, autoplay);

    const onFullscreen = () => (fullscreen = document.fullscreenElement !== null);
    const onPageHide = () => {
      finalSave();
      engine.dispose(true);
    };
    // A page restored from the back/forward cache has no live session.
    const onPageShow = (event: PageTransitionEvent) => event.persisted && location.reload();
    document.addEventListener("fullscreenchange", onFullscreen);
    window.addEventListener("pagehide", onPageHide);
    window.addEventListener("pageshow", onPageShow);
    return () => {
      document.removeEventListener("fullscreenchange", onFullscreen);
      window.removeEventListener("pagehide", onPageHide);
      window.removeEventListener("pageshow", onPageShow);
      finalSave();
      idle.stop();
      engine.dispose(true);
    };
  });

  function finalSave() {
    if (started && settled) reporter.flush(engine.absolute, duration || null, video?.ended ?? false);
  }

  function toggle() {
    if (locked || engine.problem) return;
    if (engine.paused) {
      void engine.play();
      onintent?.({ type: "play" });
    } else {
      engine.pause();
      onintent?.({ type: "pause" });
    }
  }

  function seek(target: number) {
    if (locked || !duration) return;
    const position = Math.min(Math.max(target, 0), Math.max(duration - 0.5, 0));
    engine.seek(position);
    onintent?.({ type: "seek", position });
  }

  function toggleFullscreen() {
    if (document.fullscreenElement) {
      void document.exitFullscreen();
    } else if (stage?.requestFullscreen) {
      void stage.requestFullscreen().catch(() => undefined);
    } else {
      // iPhone Safari only supports native fullscreen on the video element.
      (video as HTMLVideoElement & { webkitEnterFullscreen?: () => void })?.webkitEnterFullscreen?.();
    }
  }

  function setVolume(value: number) {
    if (!video) return;
    video.volume = Math.min(Math.max(value, 0), 1);
    video.muted = video.volume === 0;
  }

  function onTimeUpdate() {
    if (!video) return;
    engine.mediaTime = video.currentTime;
    if (!paused && settled) reporter.tick(engine.absolute, duration || null);
  }

  function onKeydown(event: KeyboardEvent) {
    if (!video || engine.problem) return;
    const action = playerShortcut(event);
    if (!action) return;
    if (locked && (action.type === "toggle" || action.type === "skip")) return;
    event.preventDefault();
    idle.poke();
    switch (action.type) {
      case "toggle":
        return toggle();
      case "skip":
        return seek(current + action.seconds);
      case "volume":
        return setVolume(video.volume + action.delta);
      case "mute":
        video.muted = !video.muted;
        return;
      case "fullscreen":
        return toggleFullscreen();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div
  bind:this={stage}
  class={cn("fixed inset-0 bg-black text-white select-none", !controlsVisible && "cursor-none")}
  role="region"
  aria-label="Video player"
  onpointermove={idle.poke}
  onpointerdown={idle.poke}
>
  <!-- Subtitles are not supported yet. -->
  <video
    bind:this={video}
    bind:duration={mediaDuration}
    bind:paused
    bind:muted
    bind:volume
    bind:buffered
    preload="metadata"
    playsinline
    class="size-full object-contain"
    onplay={() => (started = true)}
    onwaiting={() => (waiting = true)}
    onplaying={() => (waiting = false)}
    oncanplay={() => (waiting = false)}
    ontimeupdate={onTimeUpdate}
    onpause={() => settled && reporter.save(engine.absolute, duration || null)}
    onseeked={() => {
      if (video) engine.mediaTime = video.currentTime;
      if (started && settled) reporter.save(engine.absolute, duration || null);
    }}
    onended={() => reporter.save(engine.absolute, duration || null, true)}
    onerror={() => void engine.handleMediaError()}
    onclick={toggle}
    ondblclick={toggleFullscreen}
  ></video>

  {#if engine.problem}
    <PlaybackFailure
      title={engine.problem.title}
      detail={engine.problem.detail}
      {backHref}
      onretry={engine.problem.signedOut
        ? undefined
        : // A forced delivery that failed is retried automatically chosen.
          () => (engine.mode === "auto" ? engine.retry() : engine.setMode("auto"))}
    />
  {:else if engine.phase === "starting" || (waiting && !paused)}
    <div class="pointer-events-none absolute inset-0 grid place-items-center" role="status">
      <div class="flex flex-col items-center gap-3">
        <Spinner class="size-8 text-white/80" />
        {#if engine.phase === "starting"}
          <span class="text-sm text-white/70">Preparing stream</span>
        {/if}
      </div>
    </div>
  {:else if paused && (!started || engine.autoplayBlocked) && !locked}
    <div class="pointer-events-none absolute inset-0 grid place-items-center">
      <Button
        size="icon-lg"
        class="pointer-events-auto size-16 rounded-full [&_svg:not([class*='size-'])]:size-7"
        aria-label="Play"
        onclick={toggle}
      >
        <PlayIcon />
      </Button>
    </div>
  {/if}

  {@render aside?.()}

  <div
    class={cn(
      "pointer-events-none absolute inset-0 flex flex-col justify-between transition-opacity duration-300",
      controlsVisible ? "opacity-100" : "opacity-0",
    )}
  >
    <!-- Scrims keep white controls legible over bright frames. -->
    <div
      class="pointer-events-auto flex items-start gap-3 bg-linear-to-b from-black/80 to-transparent px-4 pt-[max(1rem,env(safe-area-inset-top))] pb-10 sm:px-6"
    >
      <Button href={backHref} variant="ghost" size="icon-lg" aria-label="Back">
        <ArrowLeftIcon />
      </Button>
      <div class="flex min-w-0 flex-1 flex-col pt-1">
        <span class="truncate font-medium">{item.kind === "episode" ? episodeName(item) : item.title}</span>
        {#if subtitle}<span class="truncate text-xs text-white/60">{subtitle}</span>{/if}
        {#if engine.notice}<span class="text-xs text-white/70" role="status">{engine.notice}</span>{/if}
      </div>
      {@render status?.()}
      {#if saveFailed}
        <span class="flex shrink-0 items-center gap-1.5 pt-1.5 text-xs text-white/70" role="status">
          <CloudAlertIcon class="size-4" />
          Progress not saved
        </span>
      {/if}
    </div>
    {#if !engine.problem}
      <div
        class="pointer-events-auto bg-linear-to-t from-black/85 to-transparent px-4 pt-12 pb-[max(0.75rem,env(safe-area-inset-bottom))] sm:px-6"
      >
        <PlayerControls
          {paused}
          {current}
          {duration}
          {bufferedEnd}
          {muted}
          {volume}
          {fullscreen}
          {locked}
          ontoggle={toggle}
          onseek={seek}
          onskip={(delta) => seek(current + delta)}
          onmute={() => video && (video.muted = !video.muted)}
          onvolume={setVolume}
          onfullscreen={toggleFullscreen}
        >
          {#snippet extra()}
            <DeliveryMenu {engine} {canTranscode} />
          {/snippet}
        </PlayerControls>
      </div>
    {/if}
  </div>
</div>
