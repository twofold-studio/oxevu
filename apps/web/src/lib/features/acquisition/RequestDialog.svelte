<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { errorMessage } from "$lib/api/client";
  import type { DiscoverTitle, CatalogEpisode } from "$lib/api/acquisition-types";
  import { emptyScope } from "./format";
  import RequestScope from "./RequestScope.svelte";
  let { open = $bindable(), title, episodes, onsent }: { open: boolean; title: DiscoverTitle; episodes: CatalogEpisode[]; onsent: () => void } = $props();
  let scope = $state(emptyScope());let pending = $state(false);let error = $state<string | null>(null);
  async function submit(event: SubmitEvent) {
    event.preventDefault();pending = true;error = null;
    try { await acquisitionApi.request(title, $state.snapshot(scope)); open = false; onsent(); }
    catch (e) { error = errorMessage(e); } finally { pending = false; }
  }
</script>
<Dialog.Root bind:open><Dialog.Content>
  <Dialog.Header><Dialog.Title>Request {title.title}</Dialog.Title><Dialog.Description>Your request goes to an administrator for approval. Existing library access remains unchanged.</Dialog.Description></Dialog.Header>
  <form onsubmit={submit} class="flex flex-col gap-5">
    <RequestScope bind:scope {title} {episodes} />
    {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
    <Dialog.Footer><Button variant="outline" onclick={() => open = false}>Cancel</Button><Button type="submit" disabled={pending}>{#if pending}<Spinner data-icon="inline-start" />{/if}Request {title.kind === "movie" ? "movie" : "episodes"}</Button></Dialog.Footer>
  </form>
</Dialog.Content></Dialog.Root>
