<script lang="ts">
  import { onMount } from "svelte";
  import SearchIcon from "@lucide/svelte/icons/search";
  import FilmIcon from "@lucide/svelte/icons/film";
  import TvIcon from "@lucide/svelte/icons/tv";
  import * as InputGroup from "$lib/components/ui/input-group";
  import { Spinner } from "$lib/components/ui/spinner";
  import { catalog } from "$lib/state/catalog.svelte";
  import { cn } from "$lib/utils";
  import { itemLabel, searchItems } from "./item-label";

  // Picks a title the current user can access. The server additionally
  // checks that every party member can open it.
  let { value = $bindable(), invalid = false }: { value: string | null; invalid?: boolean } = $props();

  let query = $state("");
  const results = $derived(searchItems(catalog.items, query));
  const selected = $derived(value ? catalog.find(value) : undefined);

  onMount(() => {
    void catalog.ensure();
  });
</script>

<div class="flex flex-col gap-2">
  <InputGroup.Root>
    <InputGroup.Addon><SearchIcon /></InputGroup.Addon>
    <InputGroup.Input bind:value={query} placeholder="Search titles" aria-label="Search titles" aria-invalid={invalid || undefined} />
  </InputGroup.Root>
  {#if catalog.status === "loading" || catalog.status === "idle"}
    <p class="flex items-center gap-2 text-sm text-muted-foreground"><Spinner /> Loading titles</p>
  {:else if catalog.status === "error"}
    <p class="text-sm text-destructive">{catalog.error}</p>
  {:else if results.length === 0}
    <p class="text-sm text-muted-foreground">No matching titles.</p>
  {:else}
    <ul class="flex max-h-60 flex-col overflow-y-auto rounded-lg border" role="listbox" aria-label="Titles">
      {#each results as item (item.id)}
        <li role="option" aria-selected={value === item.id}>
          <button
            type="button"
            class={cn(
              "flex w-full items-center gap-2 px-3 py-2 text-left text-sm transition-colors hover:bg-accent/50",
              value === item.id && "bg-accent text-accent-foreground",
            )}
            onclick={() => (value = item.id)}
          >
            {#if item.kind === "movie"}<FilmIcon class="size-4 shrink-0 text-muted-foreground" />{:else}<TvIcon class="size-4 shrink-0 text-muted-foreground" />{/if}
            <span class="truncate">{itemLabel(item)}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
  {#if selected}
    <p class="truncate text-xs text-muted-foreground">Selected: <span class="text-foreground">{itemLabel(selected)}</span></p>
  {/if}
</div>
