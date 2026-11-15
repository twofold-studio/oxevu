<script lang="ts">
  import CaptionsIcon from "@lucide/svelte/icons/captions";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { buttonVariants } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import SubtitleSearchDialog from "./SubtitleSearchDialog.svelte";
  import type { Subtitles } from "$lib/playback/subtitles.svelte";

  let { subtitles, itemId, canDownload, open = $bindable(false) }: { subtitles: Subtitles; itemId: string; canDownload: boolean; open?: boolean } = $props();
  let query = $state("");
  const searchable = $derived(subtitles.tracks.length > 8);
  const filtered = $derived(subtitles.tracks.filter((track) =>
    !searchable || `${track.label} ${track.language ?? ""} ${track.codec}`.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()),
  ));
</script>

<DropdownMenu.Root bind:open>
  <DropdownMenu.Trigger class={buttonVariants({ variant: "ghost", size: "icon-lg" })} aria-label="Subtitles">
    <CaptionsIcon />
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end" side="top" class="w-72 max-h-96 overflow-y-auto" portalProps={{ disabled: true }}>
    {#if searchable}
      <div class="sticky top-0 bg-popover p-1">
        <Input bind:value={query} type="search" aria-label="Search subtitle tracks"
          placeholder="Search languages or tracks…"
          onkeydown={(event) => { if (event.key !== "Escape" && event.key !== "Tab") event.stopPropagation(); }} />
      </div>
    {/if}
    <DropdownMenu.RadioGroup value={subtitles.selected} onValueChange={(value) => subtitles.select(value)}>
      <DropdownMenu.GroupHeading>Subtitles</DropdownMenu.GroupHeading>
      <DropdownMenu.RadioItem value="off">Off</DropdownMenu.RadioItem>
      {#each filtered as track (track.id)}
        <DropdownMenu.RadioItem value={track.id} disabled={!track.supported}>
          <span>{track.label}{track.forced ? " · Forced" : ""}{!track.supported ? " · Not supported" : ""}</span>
        </DropdownMenu.RadioItem>
      {/each}
    </DropdownMenu.RadioGroup>
    {#if searchable && query.trim() && filtered.length === 0}
      <DropdownMenu.Label class="text-xs font-normal text-muted-foreground">No matching subtitle tracks.</DropdownMenu.Label>
    {/if}
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
