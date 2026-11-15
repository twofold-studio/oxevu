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
  import ItemPicker from "./ItemPicker.svelte";

  // Parent remounts per opening; `itemId` preselects a title.
  let { open = $bindable(false), itemId = null }: { open: boolean; itemId?: string | null } = $props();

  // svelte-ignore state_referenced_locally
  let selected = $state<string | null>(itemId);
  let name = $state(`${session.user?.username ?? "My"}'s party`);
  let submitted = $state(false);
  let pending = $state(false);
  let failure = $state<string | null>(null);

  const errors = $derived({
    name: !name.trim() ? "Enter a name." : name.trim().length > 80 ? "Use at most 80 characters." : null,
    item: selected ? null : "Choose a title.",
  });

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    submitted = true;
    failure = null;
    if (errors.name || errors.item || pending || !selected) return;
    pending = true;
    try {
      const { party } = await partiesApi.create({ name: name.trim(), item_id: selected });
      open = false;
      router.go(href.party(party.id));
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
        You host: play, pause, seeking and title changes apply to everyone. Each guest streams on their own
        connection, so sync is close but not frame-perfect.
      </Dialog.Description>
    </Dialog.Header>
    <form onsubmit={submit} novalidate>
      <Field.Group>
        <Field.Field data-invalid={submitted && errors.name ? true : undefined}>
          <Field.Label for="party-name">Name</Field.Label>
          <Input id="party-name" bind:value={name} aria-invalid={submitted && errors.name ? true : undefined} />
          {#if submitted && errors.name}<Field.Error>{errors.name}</Field.Error>{/if}
        </Field.Field>
        <Field.Field data-invalid={submitted && errors.item ? true : undefined}>
          <Field.Label>Title</Field.Label>
          <ItemPicker bind:value={selected} invalid={submitted && Boolean(errors.item)} />
          {#if submitted && errors.item}<Field.Error>{errors.item}</Field.Error>{/if}
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
