<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import type { SearchState } from "./release-search";
  let { searchState, oncancel, onretry }: { searchState: SearchState; oncancel: () => void; onretry: () => void } = $props();
  let now = $state(Date.now());
  onMount(() => { const timer = setInterval(() => now = Date.now(), 1000); return () => clearInterval(timer); });
</script>

{#if searchState.kind === "searching"}
  <div class="flex items-center justify-between gap-3" role="status">
    <p class="flex items-center gap-2 text-sm text-muted-foreground"><Spinner />Searching {searchState.total || "configured"} indexers… {(Math.max(0, now - searchState.startedAt) / 1000).toFixed(1)} s</p>
    <Button size="sm" variant="outline" onclick={oncancel}>Cancel search</Button>
  </div>
{:else if searchState.kind === "failed"}
  <div class="flex flex-col gap-3">
    <p role="alert" class="text-sm text-destructive">{searchState.message}</p>
    <Button size="sm" variant="outline" class="self-start" onclick={onretry}>Retry search</Button>
    <details class="text-xs text-muted-foreground"><summary class="cursor-pointer">Search details · {(searchState.elapsed / 1000).toFixed(1)} s</summary>
      {#each searchState.result?.errors ?? [] as error (error.indexer_id)}<p class="mt-2">{error.indexer_name}: {error.error} · {error.retryable ? "Can retry" : "Check configuration"}</p>{/each}
    </details>
  </div>
{:else if searchState.kind === "results" || searchState.kind === "empty"}
  <p class="text-xs text-muted-foreground" role="status">{searchState.result.releases.length} unique torrents from {searchState.result.searched.hits} hits · {searchState.result.searched.indexers - searchState.result.searched.failed} of {searchState.result.searched.indexers} indexers · {(searchState.result.searched.took_ms / 1000).toFixed(1)} s</p>
{/if}
