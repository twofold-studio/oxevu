import { onMount } from "svelte";
import { integrationsApi, type Provider, type ProviderUpdate } from "$lib/api/integrations";
import { errorMessage } from "$lib/api/client";
import { createResource } from "../resource.svelte";

// Provider updates are partial on the server, so pages never overwrite each
// other's providers. One app-wide queue keeps them ordered and lets a page
// opened mid-save load after the save lands.
let queue: Promise<unknown> = Promise.resolve();

/** Integration state and per-provider saving for one page. Call during component init. */
export function createIntegrations() {
  const resource = createResource(async () => {
    await queue;
    return integrationsApi.get();
  });
  let status = $state("Changes save automatically when you finish editing a field.");
  let failure = $state(false);
  let revision = 0;

  function update(provider: Provider | "studio_images", patch: ProviderUpdate) {
    const current = ++revision;
    status = "Saving changes…";
    failure = false;
    queue = queue.then(async () => {
      try {
        resource.value = await integrationsApi.update({ [provider]: patch });
        if (revision === current) status = "Changes saved.";
      } catch (error) {
        if (revision === current) {
          status = `Not saved: ${errorMessage(error)}. Edit the field to retry.`;
          failure = true;
        }
      }
    });
  }

  onMount(() => {
    void resource.load();
  });

  return {
    resource,
    update,
    get status() {
      return status;
    },
    get failure() {
      return failure;
    },
  };
}
