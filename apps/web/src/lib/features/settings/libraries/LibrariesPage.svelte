<script lang="ts">
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import * as Empty from "$lib/components/ui/empty";
  import { Button } from "$lib/components/ui/button";
  import { librariesApi } from "$lib/api/endpoints";
  import type { Library } from "$lib/api/types";
  import { catalog } from "$lib/state/catalog.svelte";
  import ResourceGate from "../ResourceGate.svelte";
  import SettingsFrame from "../layout/SettingsFrame.svelte";
  import { createResource } from "../resource.svelte";
  import LibraryRow from "./LibraryRow.svelte";
  import LibraryDialog from "./LibraryDialog.svelte";
  import RemoveLibraryDialog from "./RemoveLibraryDialog.svelte";

  const libraries = createResource(async () => (await librariesApi.list()).libraries);

  let editorOpen = $state(false);
  let editing = $state<Library | null>(null);
  let removeOpen = $state(false);
  let removing = $state<Library | null>(null);
  // Dialogs remount per opening so they start from fresh state.
  let opening = $state(0);

  onMount(() => {
    void libraries.load();
  });

  function openEditor(library: Library | null) {
    editing = library;
    opening++;
    editorOpen = true;
  }

  function saved(library: Library, created: boolean) {
    const list = libraries.value ?? [];
    libraries.value = created ? [...list, library] : list.map((l) => (l.id === library.id ? library : l));
    if (created) toast.success(`Added ${library.name}. Scan it to list its titles.`);
    // Enabled state and roots change what the catalog shows.
    if (!created) void catalog.ensure(true);
  }

  function removed(library: Library) {
    libraries.value = (libraries.value ?? []).filter((l) => l.id !== library.id);
    toast.success(`Removed ${library.name}. Its files were not touched.`);
    void catalog.ensure(true);
  }
</script>

<SettingsFrame
  page="media/libraries"
  description="Movie and TV folders Oxevu scans. It only reads them; it never renames, moves or deletes files. Scans run when you start them."
>
  {#snippet actions()}
    {#if libraries.status === "ready" && (libraries.value ?? []).length > 0}
      <Button size="sm" onclick={() => openEditor(null)}>
        <PlusIcon data-icon="inline-start" />
        Add library
      </Button>
    {/if}
  {/snippet}

  <ResourceGate status={libraries.status} error={libraries.error} what="libraries" onretry={libraries.load}>
    {#if (libraries.value ?? []).length === 0}
      <Empty.Root class="border">
        <Empty.Header>
          <Empty.Media variant="icon"><FolderOpenIcon /></Empty.Media>
          <Empty.Title>No libraries yet</Empty.Title>
          <Empty.Description>Add a movie or TV library with one or more folders, then scan it.</Empty.Description>
        </Empty.Header>
        <Empty.Content>
          <Button onclick={() => openEditor(null)}>
            <PlusIcon data-icon="inline-start" />
            Add library
          </Button>
        </Empty.Content>
      </Empty.Root>
    {:else}
      <ul class="divide-y rounded-xl border bg-card">
        {#each libraries.value ?? [] as library (library.id)}
          <LibraryRow
            {library}
            onedit={() => openEditor(library)}
            onremove={() => {
              removing = library;
              opening++;
              removeOpen = true;
            }}
          />
        {/each}
      </ul>
    {/if}
  </ResourceGate>
</SettingsFrame>

{#key opening}
  <LibraryDialog bind:open={editorOpen} library={editing} onsaved={saved} />
  {#if removing}
    <RemoveLibraryDialog bind:open={removeOpen} library={removing} onremoved={removed} />
  {/if}
{/key}
