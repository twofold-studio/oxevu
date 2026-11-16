<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import * as Field from "$lib/components/ui/field";
  import * as InputGroup from "$lib/components/ui/input-group";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { errorMessage } from "$lib/api/client";
  import { Autosave } from "$lib/features/settings/autosave";
  import type { SavePhase } from "$lib/features/settings/settings-form.svelte";
  import { validateTransferLimit } from "./activity";

  let { id, initial }: { id: string; initial: number } = $props();
  let value = $state<number | undefined>(untrack(() => initial));
  let phase = $state<SavePhase>("idle");
  let failure = $state<string | null>(null);
  const error = $derived(validateTransferLimit(value));
  const autosave = new Autosave<number>(
    async (limit) => { await acquisitionApi.downloadLimit(id, limit); },
    (state, cause) => { phase = state; failure = cause ? errorMessage(cause) : null; },
  );
  autosave.reset(untrack(() => initial));

  function edit(event: Event) {
    const next = (event.currentTarget as HTMLInputElement).valueAsNumber;
    value = next;
    const invalid = validateTransferLimit(next);
    autosave.update(invalid ? null : next);
    if (autosave.queued) { phase = "pending"; failure = null; }
  }
  onDestroy(() => { void autosave.flush(); });
</script>

<Field.Field orientation="horizontal" class="flex-wrap gap-x-5" data-invalid={error ? true : undefined}>
  <div class="flex w-48 flex-col gap-2">
    <Field.Label for={`download-limit-${id}`}>Entry download limit</Field.Label>
    <InputGroup.Root>
      <InputGroup.Input id={`download-limit-${id}`} type="number" inputmode="numeric"
        min={0} max={1000000} step={1} {value} oninput={edit} onblur={() => void autosave.flush()}
        aria-invalid={error ? true : undefined} aria-describedby={`download-limit-help-${id}`} />
      <InputGroup.Addon align="inline-end"><InputGroup.Text>KiB/s</InputGroup.Text></InputGroup.Addon>
    </InputGroup.Root>
  </div>
  <Field.Content class="min-w-40 max-w-lg">
    <Field.Description id={`download-limit-help-${id}`}>Zero removes this entry's cap; the combined cap still applies. Changes briefly reconnect this transfer.</Field.Description>
    {#if error}<Field.Error>{error}</Field.Error>{/if}
    <p role="status" class="text-xs text-muted-foreground">
      {#if failure}<span class="text-destructive">Not saved: {failure}. Change the value to retry.</span>
      {:else if phase === "pending" || phase === "saving"}Saving…
      {:else if phase === "saved"}Limit saved.
      {:else}Changes save automatically.{/if}
    </p>
  </Field.Content>
</Field.Field>
