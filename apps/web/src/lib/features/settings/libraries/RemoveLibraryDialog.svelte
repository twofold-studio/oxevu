<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Spinner } from "$lib/components/ui/spinner";
  import { librariesApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import type { Library } from "$lib/api/types";

  let {
    open = $bindable(false),
    library,
    onremoved,
  }: { open: boolean; library: Library; onremoved: (library: Library) => void } = $props();

  let pending = $state(false);
  let failure = $state<string | null>(null);

  async function remove(event: Event) {
    // Keep the dialog open until the server confirms.
    event.preventDefault();
    pending = true;
    failure = null;
    try {
      await librariesApi.remove(library.id);
      open = false;
      onremoved(library);
    } catch (error) {
      failure = errorMessage(error);
    } finally {
      pending = false;
    }
  }
</script>

<AlertDialog.Root bind:open>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Remove {library.name}?</AlertDialog.Title>
      <AlertDialog.Description>
        Oxevu stops listing this library's titles and removes its configuration. The media files stay where
        they are and are not modified.
      </AlertDialog.Description>
    </AlertDialog.Header>
    {#if failure}<p class="text-sm text-destructive" role="alert">{failure}</p>{/if}
    <AlertDialog.Footer>
      <AlertDialog.Cancel disabled={pending}>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action variant="destructive" onclick={remove} disabled={pending}>
        {#if pending}<Spinner data-icon="inline-start" />{/if}
        Remove library
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
