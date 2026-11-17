<script lang="ts">
  import { untrack } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { CatalogEpisode, MonitoredTitle, Monitor } from "$lib/api/acquisition-types";
  import { errorMessage } from "$lib/api/client";
  import RequestScope from "./RequestScope.svelte";

  let { open = $bindable(), entry, episodes, onsaved }: {
    open: boolean;
    entry: MonitoredTitle;
    episodes: CatalogEpisode[];
    onsaved?: (monitor: Monitor) => void;
  } = $props();

  let scope = $state(untrack(() => structuredClone($state.snapshot(entry.monitor.scope))));
  let saving = $state(false);
  let error = $state<string | null>(null);
  const empty = $derived(
    (scope.mode === "seasons" && scope.seasons.length === 0) ||
    (scope.mode === "episodes" && scope.episodes.length === 0),
  );

  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (saving || empty) return;
    saving = true;
    error = null;
    try {
      const { monitor } = await acquisitionApi.monitor(entry.title.id, {
        ...entry.monitor, scope: $state.snapshot(scope),
      });
      onsaved?.(monitor);
      open = false;
    } catch (failure) {
      error = errorMessage(failure);
    } finally {
      saving = false;
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Download selection · {entry.title.title}</Dialog.Title>
      <Dialog.Description>
        Choose what this series tracks in Wanted. You can change the selection here later.
        {#if entry.monitor.automatic}This also updates the scope of automatic searches.{/if}
      </Dialog.Description>
    </Dialog.Header>
    <form onsubmit={save} class="flex flex-col gap-5">
      <RequestScope bind:scope title={entry.title} {episodes} monitoring />
      {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
      <Dialog.Footer>
        <Button variant="outline" disabled={saving} onclick={() => open = false}>Cancel</Button>
        <Button type="submit" disabled={saving || empty}>{saving ? "Saving…" : "Save selection"}</Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
