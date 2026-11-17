<script lang="ts">
  import type { Item } from "$lib/api/types";
  import { fileFacts } from "$lib/media/file-facts";
  import { factPins } from "$lib/state/fact-pins.svelte";
  import * as Popover from "$lib/components/ui/popover";
  import { Button } from "$lib/components/ui/button";
  import PinIcon from "@lucide/svelte/icons/pin";
  import PinOffIcon from "@lucide/svelte/icons/pin-off";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";

  let { item, files }: { item: Item; files?: Item[] } = $props();
  const rows = $derived(fileFacts(item, files));
</script>

<Popover.Root>
  <Popover.Trigger class="inline-flex items-center gap-2 rounded-md py-1 text-sm text-muted-foreground hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring">
    Stats for nerds <ChevronDownIcon class="size-3.5" aria-hidden="true" />
  </Popover.Trigger>
  <Popover.Content align="end" class="max-h-[var(--bits-popover-content-available-height)] w-96 max-w-[calc(100vw-2rem)] overflow-y-auto p-0">
    <dl class="divide-y text-sm">
      {#each rows as row (row.id)}
        {@const pinned = factPins.ids.includes(row.id)}
        <div class="grid grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-3 px-3 py-2">
          <dt class="text-muted-foreground">{row.label}</dt>
          <dd class="max-w-44 truncate font-mono text-xs" title={row.value ?? "Unknown"}>{row.value ?? "Unknown"}</dd>
          <Button variant="ghost" size="icon-sm" aria-label={`${pinned ? "Unpin" : "Pin"} ${row.label.toLowerCase()}`} aria-pressed={pinned} onclick={() => factPins.toggle(row.id)}>
            {#if pinned}<PinOffIcon />{:else}<PinIcon />{/if}
          </Button>
        </div>
      {/each}
    </dl>
    <p class="border-t px-3 py-2 text-xs text-muted-foreground">Pin attributes to show them under titles. Saved for this account in this browser.</p>
  </Popover.Content>
</Popover.Root>
