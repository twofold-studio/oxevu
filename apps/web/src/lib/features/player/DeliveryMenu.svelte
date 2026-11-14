<script lang="ts">
  import SettingsIcon from "@lucide/svelte/icons/settings-2";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { buttonVariants } from "$lib/components/ui/button";
  import type { PlaybackMode } from "$lib/api/playback-types";
  import type { PlaybackEngine } from "$lib/playback/engine.svelte";
  import { methodDetail, methodLabel, modeLabel } from "$lib/playback/method";

  // Shows how the stream is delivered right now and lets the viewer force
  // the original file or conversion. Conversion is offered only when the
  // account may transcode; the server enforces it either way.
  let { engine, canTranscode }: { engine: PlaybackEngine; canTranscode: boolean } = $props();

  const modes: PlaybackMode[] = ["auto", "direct", "transcode"];
  const method = $derived(engine.session?.method ?? null);
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class={buttonVariants({ variant: "ghost", size: "icon-lg" })}
    aria-label={`Playback: ${method ? methodLabel[method] : "starting"}`}
  >
    <SettingsIcon />
  </DropdownMenu.Trigger>
  <!-- Rendered inside the player (no portal) so it stays visible in full screen. -->
  <DropdownMenu.Content align="end" side="top" class="w-72" portalProps={{ disabled: true }}>
    <DropdownMenu.Label class="flex flex-col gap-1">
      <span class="text-xs font-normal text-muted-foreground">Now playing as</span>
      <span>{method ? methodLabel[method] : "Preparing stream"}</span>
      {#if method}
        <span class="text-xs leading-5 font-normal text-muted-foreground">{methodDetail[method]}</span>
      {/if}
      {#if engine.session?.reason}
        <span class="text-xs leading-5 font-normal text-muted-foreground">{engine.session.reason}</span>
      {/if}
    </DropdownMenu.Label>
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
  </DropdownMenu.Content>
</DropdownMenu.Root>
