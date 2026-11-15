<script lang="ts">
  import CaptionsIcon from "@lucide/svelte/icons/captions";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { buttonVariants } from "$lib/components/ui/button";
  import SubtitleSearchDialog from "./SubtitleSearchDialog.svelte";
  import type { Subtitles } from "$lib/playback/subtitles.svelte";

  let { subtitles, itemId, canDownload }: { subtitles: Subtitles; itemId: string; canDownload: boolean } = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger class={buttonVariants({ variant: "ghost", size: "icon-lg" })} aria-label="Subtitles">
    <CaptionsIcon />
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end" side="top" class="w-72 max-h-96 overflow-y-auto" portalProps={{ disabled: true }}>
    <DropdownMenu.RadioGroup value={subtitles.selected} onValueChange={(value) => subtitles.select(value)}>
      <DropdownMenu.GroupHeading>Subtitles</DropdownMenu.GroupHeading>
      <DropdownMenu.RadioItem value="off">Off</DropdownMenu.RadioItem>
      {#each subtitles.tracks as track (track.id)}
        <DropdownMenu.RadioItem value={track.id} disabled={!track.supported}>
          <span>{track.label}{track.forced ? " · Forced" : ""}{!track.supported ? " · Not supported" : ""}</span>
        </DropdownMenu.RadioItem>
      {/each}
    </DropdownMenu.RadioGroup>
    {#if subtitles.loading}
      <DropdownMenu.Label class="text-xs font-normal text-muted-foreground">Loading subtitles…</DropdownMenu.Label>
    {:else if subtitles.error}
      <DropdownMenu.Label class="text-xs font-normal text-destructive">{subtitles.error}</DropdownMenu.Label>
    {:else if subtitles.tracks.length === 0}
      <DropdownMenu.Label class="text-xs font-normal text-muted-foreground">No subtitle tracks found.</DropdownMenu.Label>
    {/if}
    {#if subtitles.tracks.some((track) => !track.supported)}
      <DropdownMenu.Label class="text-xs leading-5 font-normal text-muted-foreground">Image-based subtitles require burning into video, which is not available yet.</DropdownMenu.Label>
    {/if}
    <DropdownMenu.Separator />
    <DropdownMenu.Item onclick={() => (subtitles.searchOpen = true)}>Find subtitles…</DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>

{#if subtitles.searchOpen}
  <SubtitleSearchDialog bind:open={subtitles.searchOpen} {subtitles} {itemId} {canDownload} />
{/if}
