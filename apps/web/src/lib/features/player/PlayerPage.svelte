<script lang="ts">
  import { onMount } from "svelte";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import PlayIcon from "@lucide/svelte/icons/play";
  import CloudAlertIcon from "@lucide/svelte/icons/cloud-alert";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { itemsApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import type { Item } from "$lib/api/types";
  import { catalog } from "$lib/state/catalog.svelte";
  import { session } from "$lib/state/session.svelte";
  import { href } from "$lib/state/router.svelte";
  import { episodeCode } from "$lib/media/grouping";
  import { resumePosition } from "$lib/media/progress";
  import { cn } from "$lib/utils";
  import PlayerControls from "./PlayerControls.svelte";
  import PlaybackFailure from "./PlaybackFailure.svelte";
  import { ProgressReporter } from "./progress-reporter";
  import { explainPlaybackError, type PlaybackProblem } from "./playback-error";
  import { createIdle } from "./idle.svelte";
  import { playerShortcut } from "./shortcuts";

  let { id, fromStart }: { id: string; fromStart: boolean } = $props();

  let item = $state<Item | null>(null);
  let loadError = $state<string | null>(null);
  let problem = $state<PlaybackProblem | null>(null);

  let stage = $state<HTMLDivElement | null>(null);
  let video = $state<HTMLVideoElement | null>(null);
  let current = $state(0);
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
  const streamUrl = $derived(itemsApi.streamUrl(id));
  // Prefer the browser's measured duration; fall back to the probe value.
  const duration = $derived(
    Number.isFinite(mediaDuration) && mediaDuration > 0 ? mediaDuration : (item?.duration_seconds ?? 0),
  );
  const bufferedEnd = $derived(buffered.find((r) => r.start <= current + 0.5 && r.end >= current)?.end ?? 0);
  const controlsVisible = $derived(!idle.idle || paused || problem !== null);
  const subtitle = $derived(
    item?.kind === "episode" ? [item.show_title, episodeCode(item)].filter(Boolean).join("  ") : null,
  );

  // App.svelte keys this component by id, so capturing the initial id is intended.
  // svelte-ignore state_referenced_locally
  const reporter = new ProgressReporter(
    id,
    (progress) => {
      saveFailed = false;
      catalog.patch(id, progress);
    },
    () => (saveFailed = true),
  );

  onMount(() => {
    // Always refetch: resume must use the latest saved position, which may
    // come from another device.
    itemsApi
      .get(id)
      .then(({ item: fresh }) => {
        item = fresh;
        catalog.patch(id, fresh);
      })
      .catch((error: unknown) => (loadError = errorMessage(error)));

    const onFullscreen = () => (fullscreen = document.fullscreenElement !== null);
    const onPageHide = () => finalSave();
    document.addEventListener("fullscreenchange", onFullscreen);
    window.addEventListener("pagehide", onPageHide);
    return () => {
      document.removeEventListener("fullscreenchange", onFullscreen);
      window.removeEventListener("pagehide", onPageHide);
      finalSave();
      idle.stop();
      // Release the connection right away instead of waiting for GC.
      video?.pause();
      video?.removeAttribute("src");
      video?.load();
    };
  });

  function finalSave() {
    if (video && started) reporter.flush(video.currentTime, duration || null, video.ended);
  }

  function onLoadedMetadata() {
    if (!video || !item) return;
    const resume = fromStart ? null : resumePosition(item);
    if (resume !== null && resume < video.duration) video.currentTime = resume;
    void play();
  }

  async function play() {
    try {
      await video?.play();
    } catch {
      // Autoplay was blocked; the large play button invites a click.
    }
  }

  function toggle() {
    if (!video || problem) return;
    if (video.paused) void play();
    else video.pause();
  }

  function seek(time: number) {
    if (!video || !duration) return;
    video.currentTime = Math.min(Math.max(time, 0), Math.max(duration - 0.5, 0));
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

  async function onError() {
    if (!video || !item) return;
    // A failed load from a stale tab often means the session ended.
    problem = await explainPlaybackError(video.error, item, streamUrl);
    if (problem.signedOut) void session.load();
  }

  function retry() {
    if (!video) return;
    const at = video.currentTime;
    problem = null;
    video.load();
    video.addEventListener("loadedmetadata", () => seek(at), { once: true });
  }

  function onKeydown(event: KeyboardEvent) {
    if (!video || problem) return;
    const action = playerShortcut(event);
    if (!action) return;
    event.preventDefault();
    idle.poke();
    switch (action.type) {
      case "toggle":
        return toggle();
      case "skip":
        return seek(video.currentTime + action.seconds);
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

<svelte:head>
  <title>{item ? `${item.title} · Oxevu` : "Oxevu"}</title>
</svelte:head>

<svelte:window onkeydown={onKeydown} />

<div
  bind:this={stage}
  class={cn("fixed inset-0 bg-black text-white select-none", !controlsVisible && "cursor-none")}
  role="region"
  aria-label="Video player"
  onpointermove={idle.poke}
  onpointerdown={idle.poke}
>
  {#if loadError}
    <PlaybackFailure
      title="Couldn't open this title"
      detail={loadError}
      backHref={href.home()}
    />
  {:else if item}
    <!-- svelte-ignore a11y_media_has_caption (phase one has no subtitle support) -->
    <video
      bind:this={video}
      bind:currentTime={current}
      bind:duration={mediaDuration}
      bind:paused
      bind:muted
      bind:volume
      bind:buffered
      src={streamUrl}
      preload="metadata"
      playsinline
      class="size-full object-contain"
      onloadedmetadata={onLoadedMetadata}
      onplay={() => (started = true)}
      onwaiting={() => (waiting = true)}
      onplaying={() => (waiting = false)}
      oncanplay={() => (waiting = false)}
      ontimeupdate={() => !paused && reporter.tick(current, duration || null)}
      onpause={() => reporter.save(current, duration || null)}
      onseeked={() => started && reporter.save(current, duration || null)}
      onended={() => reporter.save(current, duration || null, true)}
      onerror={onError}
      onclick={toggle}
      ondblclick={toggleFullscreen}
    ></video>

    {#if problem}
      <PlaybackFailure
        title={problem.title}
        detail={problem.detail}
        backHref={href.item(item.id)}
        onretry={problem.signedOut ? undefined : retry}
      />
    {:else if waiting && !paused}
      <div class="pointer-events-none absolute inset-0 grid place-items-center">
        <Spinner class="size-8 text-white/80" />
      </div>
    {:else if paused && !started}
      <div class="pointer-events-none absolute inset-0 grid place-items-center">
        <Button
          size="icon-lg"
          class="pointer-events-auto size-16 rounded-full [&_svg:not([class*='size-'])]:size-7"
          aria-label="Play"
          onclick={() => void play()}
        >
          <PlayIcon />
        </Button>
      </div>
    {/if}

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
        <Button href={href.item(item.id)} variant="ghost" size="icon-lg" aria-label="Back to details">
          <ArrowLeftIcon />
        </Button>
        <div class="flex min-w-0 flex-1 flex-col pt-1">
          <span class="truncate font-medium">{item.title}</span>
          {#if subtitle}<span class="truncate font-mono text-xs text-white/60">{subtitle}</span>{/if}
        </div>
        {#if saveFailed}
          <span class="flex shrink-0 items-center gap-1.5 pt-1.5 text-xs text-white/70" role="status">
            <CloudAlertIcon class="size-4" />
            Progress not saved
          </span>
        {/if}
      </div>
      {#if !problem}
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
            ontoggle={toggle}
            onseek={seek}
            onskip={(delta) => seek(current + delta)}
            onmute={() => video && (video.muted = !video.muted)}
            onvolume={setVolume}
            onfullscreen={toggleFullscreen}
          />
        </div>
      {/if}
    </div>
  {:else}
    <div class="grid size-full place-items-center" aria-busy="true">
      <Spinner class="size-8 text-white/80" />
    </div>
  {/if}
</div>
