<script lang="ts">
  import SettingsIcon from "@lucide/svelte/icons/settings-2";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { buttonVariants } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { qualityPresets, type QualityId } from "$lib/playback/quality";
  import type { PlaybackMode } from "$lib/api/playback-types";
  import type { PlaybackEngine } from "$lib/playback/engine.svelte";
  import { methodDetail, methodLabel, modeLabel } from "$lib/playback/method";

  // Shows how the stream is delivered right now and lets the viewer force
  // the original file or conversion. Conversion is offered only when the
  // account may transcode; the server enforces it either way.
  let { engine, canTranscode, open = $bindable(false) }: { engine: PlaybackEngine; canTranscode: boolean; open?: boolean } = $props();

  const modes: PlaybackMode[] = ["auto", "direct", "transcode"];
  const method = $derived(engine.session?.method ?? null);
  const currentQuality = $derived(engine.session?.method === "direct" ? "Original" :
    engine.session?.max_height && engine.session.video_bitrate_mbps
      ? `${engine.session.max_height}p · ${engine.session.video_bitrate_mbps} Mbps` : "Preparing…");
  const currentPreset = $derived(engine.session?.method === "direct" ? undefined :
    qualityPresets.find((preset) => preset.max_height === engine.session?.max_height &&
      preset.video_bitrate_mbps === engine.session?.video_bitrate_mbps)?.id);
</script>

<DropdownMenu.Root bind:open>
  <DropdownMenu.Trigger
    class={buttonVariants({ variant: "ghost", size: "icon-lg" })}
    aria-label={`Playback: ${method ? methodLabel[method] : "starting"}`}
  >
    <SettingsIcon />
  </DropdownMenu.Trigger>
  <!-- Rendered inside the player (no portal) so it stays visible in full screen. -->
  <DropdownMenu.Content align="end" side="top" class="w-72 max-h-[calc(100dvh-5rem)] overflow-y-auto" portalProps={{ disabled: true }}>
    <DropdownMenu.Label class="flex flex-col gap-1">
      <span class="text-xs font-normal text-muted-foreground">Now playing as</span>
      <span>{method ? methodLabel[method] : "Preparing stream"}</span>
      {#if method}
        <span class="text-xs leading-5 font-normal text-muted-foreground">{methodDetail[method]}</span>
      {/if}
      {#if engine.session?.decoder}
        <span class="text-xs font-normal text-muted-foreground">Decoder: {engine.session.decoder === "cuda" ? "NVIDIA CUDA" : engine.session.decoder === "copy" ? "Stream copy" : "CPU"}</span>
      {/if}
      {#if engine.session?.reason}
        <span class="text-xs leading-5 font-normal text-muted-foreground">{engine.session.reason}</span>
      {/if}
      {#if engine.videoWidth && engine.videoHeight}
        <span class="text-xs font-normal text-muted-foreground">Resolution: {engine.videoWidth} × {engine.videoHeight}</span>
      {/if}
      {#if engine.session?.cache_hit}
        <span class="text-xs font-normal text-muted-foreground">Using cached conversion</span>
      {/if}
    </DropdownMenu.Label>
    {#if engine.session?.video_bitrate_mbps && method !== "direct"}
      <DropdownMenu.Label class="text-xs font-normal text-muted-foreground">
        Up to {engine.session.max_height}p · {engine.session.video_bitrate_mbps} Mbps
      </DropdownMenu.Label>
    {/if}
    <DropdownMenu.Separator />
    <DropdownMenu.RadioGroup
      value={engine.mode}
      onValueChange={(value) => engine.setMode(value as PlaybackMode)}
    >
      <DropdownMenu.GroupHeading>Delivery</DropdownMenu.GroupHeading>
      {#each modes as mode (mode)}
        <DropdownMenu.RadioItem value={mode} disabled={mode === "transcode" && !canTranscode}>
          {modeLabel[mode]}
          {#if mode === "transcode" && !canTranscode}
            <span class="ml-auto text-xs text-muted-foreground">Not allowed</span>
          {/if}
        </DropdownMenu.RadioItem>
      {/each}
    </DropdownMenu.RadioGroup>
    <DropdownMenu.Separator />
    <DropdownMenu.RadioGroup value={engine.quality} onValueChange={(value) => engine.setQuality(value as QualityId)}>
      <DropdownMenu.GroupHeading>Quality</DropdownMenu.GroupHeading>
      <DropdownMenu.RadioItem value="auto">{engine.quality === "auto" ? `Automatic · ${currentQuality}` : "Automatic"}</DropdownMenu.RadioItem>
      {#each qualityPresets as preset (preset.id)}
        <DropdownMenu.RadioItem value={preset.id} disabled={!canTranscode}>
          {preset.label}
          {#if engine.quality === "auto" && currentPreset === preset.id}<Badge variant="secondary" class="ml-auto">Current</Badge>{/if}
        </DropdownMenu.RadioItem>
      {/each}
    </DropdownMenu.RadioGroup>
    <DropdownMenu.Label class="text-xs leading-5 font-normal text-muted-foreground">
      Automatic reduces quality on slower connections. Server limits apply. Changing quality briefly restarts the stream.
    </DropdownMenu.Label>
  </DropdownMenu.Content>
</DropdownMenu.Root>
