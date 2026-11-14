<script lang="ts">
  import * as Field from "$lib/components/ui/field";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Switch } from "$lib/components/ui/switch";
  import type { Library } from "$lib/api/types";

  // Which libraries a viewer may browse and play. Administrators see
  // everything regardless, so the grants are hidden for them.
  let {
    libraries,
    allLibraries = $bindable(),
    libraryIds = $bindable(),
    isAdmin,
  }: { libraries: Library[]; allLibraries: boolean; libraryIds: string[]; isAdmin: boolean } = $props();

  function toggle(id: string, checked: boolean) {
    libraryIds = checked ? [...new Set([...libraryIds, id])] : libraryIds.filter((value) => value !== id);
  }
</script>

<Field.Set>
  <Field.Legend variant="label">Library access</Field.Legend>
  {#if isAdmin}
    <Field.Description>Administrators can access every library.</Field.Description>
  {:else}
    <Field.Field orientation="horizontal">
      <Switch id="user-all-libraries" bind:checked={allLibraries} />
      <Field.Content>
        <Field.Label for="user-all-libraries">All libraries</Field.Label>
        <Field.Description>Includes libraries added later.</Field.Description>
      </Field.Content>
    </Field.Field>
    {#if !allLibraries}
      {#if libraries.length === 0}
        <Field.Description>No libraries exist yet.</Field.Description>
      {:else}
        <div class="flex flex-col divide-y rounded-lg border">
          {#each libraries as library (library.id)}
            <Field.Field orientation="horizontal" class="px-3 py-2.5">
              <Checkbox
                id={`grant-${library.id}`}
                checked={libraryIds.includes(library.id)}
                onCheckedChange={(checked) => toggle(library.id, checked === true)}
              />
              <Field.Label for={`grant-${library.id}`} class="flex min-w-0 flex-1 items-center justify-between gap-2 font-normal">
                <span class="truncate">{library.name}</span>
                <span class="text-xs text-muted-foreground">{library.kind === "movies" ? "Movies" : "TV"}</span>
              </Field.Label>
            </Field.Field>
          {/each}
        </div>
        {#if libraryIds.length === 0}
          <Field.Description>Without a library this account sees an empty catalog.</Field.Description>
        {/if}
      {/if}
    {/if}
  {/if}
</Field.Set>
