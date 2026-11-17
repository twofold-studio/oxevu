<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { partiesApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import { href, router } from "$lib/state/router.svelte";
  import { session } from "$lib/state/session.svelte";
  import { partySession } from "$lib/party/session.svelte";

  // Parent remounts per opening; `itemId` preselects a title.
  let { open = $bindable(false), itemId = null, oncreated }: { open: boolean; itemId?: string | null; oncreated?: (id: string) => void } = $props();

  let name = $state(`${session.user?.username ?? "My"}'s party`);
  let submitted = $state(false);
  let pending = $state(false);
  let failure = $state<string | null>(null);

  const errors = $derived({
    name: !name.trim() ? "Enter a name." : name.trim().length > 80 ? "Use at most 80 characters." : null,
  });

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    submitted = true;
    failure = null;
    if (errors.name || pending) return;
    pending = true;
    try {
      const { party } = await partiesApi.create({ name: name.trim() });
      await partySession.join(party.id);
      if (itemId) await partySession.play(itemId, true);
      open = false;
      if (oncreated) oncreated(party.id);
      else if (!itemId) router.go(href.party(party.id));
    } catch (error) {
      failure = errorMessage(error);
    } finally {
      pending = false;
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Start a watch party</Dialog.Title>
      <Dialog.Description>
        Invite others, then anyone can open a title from the library. Play, pause and seeking are shared;
        volume and subtitles stay personal.
      </Dialog.Description>
    </Dialog.Header>
    <form onsubmit={submit} novalidate>
      <Field.Group>
        <Field.Field data-invalid={submitted && errors.name ? true : undefined}>
          <Field.Label for="party-name">Name</Field.Label>
          <Input id="party-name" bind:value={name} aria-invalid={submitted && errors.name ? true : undefined} />
          {#if submitted && errors.name}<Field.Error>{errors.name}</Field.Error>{/if}
        </Field.Field>
        {#if failure}<p class="text-sm text-destructive" role="alert">{failure}</p>{/if}
        <Dialog.Footer>
          <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
          <Button type="submit" disabled={pending}>
            {#if pending}<Spinner data-icon="inline-start" />{/if}
            Start party
          </Button>
        </Dialog.Footer>
      </Field.Group>
    </form>
  </Dialog.Content>
</Dialog.Root>
