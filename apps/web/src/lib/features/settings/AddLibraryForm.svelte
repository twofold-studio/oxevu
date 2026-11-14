<script lang="ts">
  import { toast } from "svelte-sonner";
  import * as Field from "$lib/components/ui/field";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { librariesApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import type { Library, LibraryKind } from "$lib/api/types";

  let { oncreated }: { oncreated: (library: Library) => void } = $props();

  let name = $state("");
  let path = $state("");
  let kind = $state<LibraryKind>("movies");
  let submitted = $state(false);
  let pending = $state(false);
  let failure = $state<string | null>(null);

  const errors = $derived({
    name: name.trim() ? null : "Enter a name.",
    path: !path.trim() ? "Enter a folder path." : !path.trim().startsWith("/") && !/^[A-Za-z]:[\\/]/.test(path.trim()) ? "Use an absolute path, such as /srv/media/movies." : null,
  });

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    submitted = true;
    failure = null;
    if (errors.name || errors.path || pending) return;
    pending = true;
    try {
      const { library } = await librariesApi.create({ name: name.trim(), path: path.trim(), kind });
      oncreated(library);
      toast.success(`Added ${library.name}. Scan it to list its titles.`);
      name = "";
      path = "";
      submitted = false;
    } catch (error) {
      failure = errorMessage(error);
    } finally {
      pending = false;
    }
  }

  const show = (message: string | null) => (submitted ? message : null);
</script>

<form onsubmit={submit} novalidate>
  <Field.Group>
    <Field.Field data-invalid={show(errors.name) ? true : undefined}>
      <Field.Label for="library-name">Name</Field.Label>
      <Input id="library-name" bind:value={name} placeholder="Movies" aria-invalid={show(errors.name) ? true : undefined} />
      {#if show(errors.name)}<Field.Error>{errors.name}</Field.Error>{/if}
    </Field.Field>

    <Field.Field data-invalid={show(errors.path) ? true : undefined}>
      <Field.Label for="library-path">Folder</Field.Label>
      <Input
        id="library-path"
        bind:value={path}
        placeholder="/srv/media/movies"
        class="font-mono"
        spellcheck={false}
        autocapitalize="none"
        aria-invalid={show(errors.path) ? true : undefined}
      />
      {#if show(errors.path)}
        <Field.Error>{errors.path}</Field.Error>
      {:else}
        <Field.Description>Read-only. Subfolders are included.</Field.Description>
      {/if}
    </Field.Field>

    <Field.Set>
      <Field.Legend variant="label">Contents</Field.Legend>
      <ToggleGroup.Root
        type="single"
        variant="outline"
        bind:value={() => kind, (v) => v && (kind = v as LibraryKind)}
        aria-label="Library contents"
      >
        <ToggleGroup.Item value="movies">Movies</ToggleGroup.Item>
        <ToggleGroup.Item value="tv">TV episodes</ToggleGroup.Item>
      </ToggleGroup.Root>
    </Field.Set>

    {#if failure}
      <p class="text-sm text-destructive" role="alert">{failure}</p>
    {/if}

    <div>
      <Button type="submit" disabled={pending}>
        {#if pending}<Spinner data-icon="inline-start" />{/if}
        Add library
      </Button>
    </div>
  </Field.Group>
</form>
