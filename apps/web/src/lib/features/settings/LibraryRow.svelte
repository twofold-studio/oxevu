<script lang="ts">
  import ScanIcon from "@lucide/svelte/icons/scan-search";
  import FilmIcon from "@lucide/svelte/icons/film";
  import TvIcon from "@lucide/svelte/icons/tv";
  import { toast } from "svelte-sonner";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Spinner } from "$lib/components/ui/spinner";
  import { librariesApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import type { Library, ScanResult } from "$lib/api/types";
  import { catalog } from "$lib/state/catalog.svelte";
  import { plural } from "$lib/media/format";

  let { library }: { library: Library } = $props();

  let scanning = $state(false);
  let result = $state<ScanResult | null>(null);
  let failure = $state<string | null>(null);
  let showErrors = $state(false);

  async function scan() {
    scanning = true;
    failure = null;
    try {
      result = await librariesApi.scan(library.id);
      showErrors = false;
      toast.success(`Scanned ${library.name}: ${plural(result.scanned, "file")}`);
      await catalog.ensure(true);
    } catch (error) {
      failure = errorMessage(error);
    } finally {
      scanning = false;
    }
  }
</script>

<li class="flex flex-col gap-3 px-5 py-4">
  <div class="flex flex-wrap items-center gap-x-4 gap-y-3">
    <div class="flex min-w-0 flex-1 flex-col gap-1">
      <div class="flex items-center gap-2">
        <span class="truncate font-medium">{library.name}</span>
        <Badge variant="outline">
          {#if library.kind === "movies"}<FilmIcon />Movies{:else}<TvIcon />TV{/if}
        </Badge>
      </div>
      <span class="truncate font-mono text-xs text-muted-foreground" title={library.path}>{library.path}</span>
    </div>
    <Button variant="outline" onclick={scan} disabled={scanning}>
      {#if scanning}<Spinner data-icon="inline-start" />{:else}<ScanIcon data-icon="inline-start" />{/if}
      {scanning ? "Scanning" : "Scan"}
    </Button>
  </div>

  <div aria-live="polite" class="text-sm">
    {#if scanning}
      <p class="text-muted-foreground">Reading files and probing media. Large folders can take a while.</p>
    {:else if failure}
      <p class="text-destructive">Scan failed: {failure}</p>
    {:else if result}
      <p class="text-muted-foreground">
        Found {plural(result.scanned, "playable file")}.
        {#if result.errors.length > 0}
          <button
            type="button"
            class="text-foreground underline underline-offset-4"
            aria-expanded={showErrors}
            onclick={() => (showErrors = !showErrors)}
          >
            {plural(result.errors.length, "file")} couldn't be read
          </button>
        {/if}
      </p>
      {#if showErrors}
        <ul class="mt-2 flex max-h-48 flex-col gap-1 overflow-y-auto rounded-lg border bg-background p-3 font-mono text-xs text-muted-foreground">
          {#each result.errors as message, index (index)}
            <li class="break-all">{message}</li>
          {/each}
        </ul>
      {/if}
    {/if}
  </div>
</li>
