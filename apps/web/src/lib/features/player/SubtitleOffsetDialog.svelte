<script lang="ts">
  import MinusIcon from "@lucide/svelte/icons/minus";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import type { Subtitles } from "$lib/playback/subtitles.svelte";

  let { open = $bindable(false), subtitles }: {
    open: boolean;
    subtitles: Subtitles;
  } = $props();

  const selectedTrack = $derived(subtitles.tracks.find((t) => t.id === subtitles.selected));
  const disabled = $derived(subtitles.selected === "off" || !selectedTrack?.supported);

  let inputValue = $derived(String(subtitles.delay));

  function adjust(delta: number) {
    subtitles.setDelay(subtitles.delay + delta);
  }

  function reset() {
    subtitles.setDelay(0);
  }

  function commitInput() {
    const parsed = Number(inputValue);
    if (inputValue.trim() !== "" && Number.isFinite(parsed)) {
      subtitles.setDelay(parsed);
    } else {
      inputValue = String(subtitles.delay);
    }
  }

  function stopPropagation(event: KeyboardEvent) {
    if (event.key !== "Escape" && event.key !== "Tab") {
      event.stopPropagation();
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content portalProps={{ disabled: true }}>
    <Dialog.Header>
      <Dialog.Title>Subtitle timing</Dialog.Title>
      <Dialog.Description>
        {#if selectedTrack}
          Adjust timing for {selectedTrack.label}. Positive shifts captions later, negative shifts them earlier. Saved in this browser for this title and track, up to ±600 seconds.
        {:else}
          Select a subtitle track first.
        {/if}
      </Dialog.Description>
    </Dialog.Header>
    <div class="flex items-center justify-center gap-2 py-4">
      <Button variant="outline" size="icon" {disabled} onclick={() => adjust(-0.1)} aria-label="Earlier by 0.1 seconds">
        <MinusIcon />
      </Button>
      <div class="relative w-28">
        <Input
          type="text"
          inputmode="decimal"
          bind:value={inputValue}
          onblur={commitInput}
          onkeydown={(e) => { stopPropagation(e); if (e.key === "Enter") commitInput(); }}
          {disabled}
          class="text-center tabular-nums"
          aria-label="Subtitle delay in seconds"
        />
        <span class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs text-muted-foreground">s</span>
      </div>
      <Button variant="outline" size="icon" {disabled} onclick={() => adjust(0.1)} aria-label="Later by 0.1 seconds">
        <PlusIcon />
      </Button>
      <Button variant="ghost" size="icon" {disabled} onclick={reset} aria-label="Reset timing to zero" title="Reset">
        <RotateCcwIcon />
      </Button>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Close</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
