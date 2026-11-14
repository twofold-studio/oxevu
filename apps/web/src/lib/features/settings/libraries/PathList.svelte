<script lang="ts">
  import PlusIcon from "@lucide/svelte/icons/plus";
  import XIcon from "@lucide/svelte/icons/x";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";

  /** Editable list of library roots; at most `max` rows. */
  let {
    paths = $bindable(),
    errors,
    listError,
    max = 16,
  }: { paths: string[]; errors: (string | null)[]; listError: string | null; max?: number } = $props();

  function add() {
    if (paths.length < max) paths = [...paths, ""];
  }

  function remove(index: number) {
    paths = paths.length === 1 ? [""] : paths.filter((_, i) => i !== index);
  }
</script>

<Field.Set>
  <Field.Legend variant="label">Folders</Field.Legend>
  <Field.Description>
    Paths as the server sees them. Subfolders are included. Oxevu only reads these folders.
  </Field.Description>
  <div class="flex flex-col gap-2">
    {#each paths as _, index (index)}
      <Field.Field data-invalid={errors[index] ? true : undefined}>
        <div class="flex gap-2">
          <Input
            bind:value={paths[index]}
            placeholder="/srv/media/movies"
            class="font-mono"
            spellcheck={false}
            autocapitalize="none"
            aria-label={`Folder ${index + 1}`}
            aria-invalid={errors[index] ? true : undefined}
          />
          <Button
            variant="ghost"
            size="icon"
            onclick={() => remove(index)}
            aria-label={`Remove folder ${index + 1}`}
            disabled={paths.length === 1 && !paths[0]}
          >
            <XIcon />
          </Button>
        </div>
        {#if errors[index]}<Field.Error>{errors[index]}</Field.Error>{/if}
      </Field.Field>
    {/each}
  </div>
  {#if listError}<Field.Error>{listError}</Field.Error>{/if}
  <div>
    <Button variant="outline" size="sm" onclick={add} disabled={paths.length >= max}>
      <PlusIcon data-icon="inline-start" />
      Add folder
    </Button>
  </div>
</Field.Set>
