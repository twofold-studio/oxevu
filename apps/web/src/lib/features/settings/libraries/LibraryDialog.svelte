<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import { Autosave } from "../autosave";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import { Input } from "$lib/components/ui/input";
  import { Switch } from "$lib/components/ui/switch";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { librariesApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import type { Library, LibraryKind } from "$lib/api/types";
  import PathList from "./PathList.svelte";
  import { draftFrom, emptyDraft, hasErrors, toInput, validateLibrary, type LibraryDraft } from "./library-form";

  // Create (no `library`) or edit one library. Saving never touches media;
  // the server validates roots and reports overlaps.
  let {
    open = $bindable(false),
    library = null,
    onsaved,
  }: { open: boolean; library?: Library | null; onsaved: (library: Library, created: boolean) => void } =
    $props();

  // The parent remounts this dialog for each opening, so the draft starts
  // from the library as it is now.
  // svelte-ignore state_referenced_locally
  let draft = $state<LibraryDraft>(library ? draftFrom(library) : emptyDraft());
  let submitted = $state(false);
  let pending = $state(false);
  let failure = $state<string | null>(null);
  let autosaveStatus = $state("Changes save automatically.");

  const errors = $derived(validateLibrary(draft));
  const show = <T,>(value: T): T | null => (submitted || library ? value : null);

  const autosave = new Autosave<ReturnType<typeof toInput>>(async (body) => {
    if (!library) return;
    const saved = (await librariesApi.update(library.id, body)).library;
    onsaved(saved, false);
  }, (state, error) => {
    pending = state === "saving";
    failure = state === "error" ? errorMessage(error) : null;
    autosaveStatus = state === "saving" ? "Saving changes…" : "Changes saved.";
  });
  untrack(() => autosave.reset(toInput($state.snapshot(draft))));
  $effect(() => {
    if (!library) return;
    const valid = !hasErrors(errors);
    const body = valid ? toInput($state.snapshot(draft)) : null;
    untrack(() => autosave.update(body));
  });
  onDestroy(() => { void autosave.flush(); });

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (library) { void autosave.flush(); return; }
    submitted = true;
    failure = null;
    if (hasErrors(errors) || pending) return;
    pending = true;
    try {
      const body = toInput(draft);
      const saved = (await librariesApi.create(body)).library;
      onsaved(saved, !library);
      open = false;
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
      <Dialog.Title>{library ? `Edit ${library.name}` : "Add a library"}</Dialog.Title>
      <Dialog.Description>
        A library groups folders of one kind. Shows with the same name across its folders are merged.
      </Dialog.Description>
    </Dialog.Header>
    <form onsubmit={submit} novalidate>
      <Field.Group>
        <Field.Field data-invalid={show(errors.name) ? true : undefined}>
          <Field.Label for="library-name">Name</Field.Label>
          <Input id="library-name" bind:value={draft.name} placeholder="Movies" aria-invalid={show(errors.name) ? true : undefined} />
          {#if show(errors.name)}<Field.Error>{errors.name}</Field.Error>{/if}
        </Field.Field>

        <Field.Set>
          <Field.Legend variant="label">Contents</Field.Legend>
          <ToggleGroup.Root
            type="single"
            variant="outline"
            bind:value={() => draft.kind, (v) => v && (draft.kind = v as LibraryKind)}
            aria-label="Library contents"
          >
            <ToggleGroup.Item value="movies">Movies</ToggleGroup.Item>
            <ToggleGroup.Item value="tv">TV episodes</ToggleGroup.Item>
          </ToggleGroup.Root>
        </Field.Set>

        <PathList
          bind:paths={draft.paths}
          errors={submitted || library ? errors.path : []}
          listError={show(errors.paths)}
        />

        <Field.Field orientation="horizontal">
          <Switch id="library-enabled" bind:checked={draft.enabled} />
          <Field.Content>
            <Field.Label for="library-enabled">Enabled</Field.Label>
            <Field.Description>Disabled libraries are hidden from everyone and skipped by scans.</Field.Description>
          </Field.Content>
        </Field.Field>

        {#if failure}<p class="text-sm text-destructive" role="alert">{failure}</p>{/if}

        <Dialog.Footer>
          {#if library}
            <p class="text-sm text-muted-foreground" role="status">{autosaveStatus}</p>
          {:else}
            <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
            <Button type="submit" disabled={pending}>
              {#if pending}<Spinner data-icon="inline-start" />{/if}Add library
            </Button>
          {/if}
        </Dialog.Footer>
      </Field.Group>
    </form>
  </Dialog.Content>
</Dialog.Root>
