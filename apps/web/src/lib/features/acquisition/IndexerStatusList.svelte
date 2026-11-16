<script lang="ts">
  import CircleAlertIcon from "@lucide/svelte/icons/circle-alert";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import type { ReleaseResults } from "$lib/api/acquisition-types";
  let { result, onretry, disabled = false }: { result: ReleaseResults; onretry: () => void; disabled?: boolean } = $props();
</script>

<ul class="flex flex-wrap gap-2" aria-label="Indexer search outcomes">
  {#each result.searched.sources as source (source.indexer_id)}
    {@const failure = result.errors.find((error) => error.indexer_id === source.indexer_id)}
    <li>
      {#if failure}
        <details class="rounded-lg border px-2 py-1 text-xs">
          <summary class="flex cursor-pointer items-center gap-1.5"><CircleAlertIcon class="size-3.5 text-destructive" />{source.indexer_name} · failed</summary>
          <p class="mt-2 max-w-64 text-muted-foreground">{failure.error}</p>
          {#if failure.retryable}<Button size="xs" variant="ghost" {disabled} onclick={onretry}>Retry search</Button>{:else}<p class="mt-1 text-muted-foreground">Check this source's configuration before retrying.</p>{/if}
        </details>
      {:else}<Badge variant="outline">{source.indexer_name} · {source.hits ?? 0}</Badge>{/if}
    </li>
  {/each}
</ul>
