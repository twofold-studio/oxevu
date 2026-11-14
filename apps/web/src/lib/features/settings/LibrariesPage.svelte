<script lang="ts">
  import { onMount } from "svelte";
  import ShieldAlertIcon from "@lucide/svelte/icons/shield-alert";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import FolderPlusIcon from "@lucide/svelte/icons/folder-plus";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import * as Alert from "$lib/components/ui/alert";
  import * as Empty from "$lib/components/ui/empty";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import { librariesApi } from "$lib/api/endpoints";
  import { ApiError, errorMessage } from "$lib/api/client";
  import type { Library } from "$lib/api/types";
  import LibraryRow from "./LibraryRow.svelte";
  import AddLibraryForm from "./AddLibraryForm.svelte";
  import SettingsSection from "./SettingsSection.svelte";

  let libraries = $state<Library[]>([]);
  let status = $state<"loading" | "ready" | "forbidden" | "error">("loading");
  let error = $state<string | null>(null);

  async function load() {
    status = "loading";
    try {
      libraries = (await librariesApi.list()).libraries;
      status = "ready";
    } catch (e) {
      if (e instanceof ApiError && e.forbidden) {
        status = "forbidden";
      } else {
        error = errorMessage(e);
        status = "error";
      }
    }
  }

  onMount(() => {
    void load();
  });
</script>

<Page class="max-w-5xl">
  <PageHeader
    title="Libraries"
    description="Folders Oxevu scans for movies and TV episodes. Oxevu only reads these folders; it never renames, moves or deletes files."
  />

  {#if status === "forbidden"}
    <Empty.Root class="border">
      <Empty.Header>
        <Empty.Media variant="icon"><ShieldAlertIcon /></Empty.Media>
        <Empty.Title>Administrator access needed</Empty.Title>
        <Empty.Description>Ask the server administrator to add or scan libraries.</Empty.Description>
      </Empty.Header>
    </Empty.Root>
  {:else if status === "error"}
    <Alert.Root variant="destructive">
      <Alert.Title>Couldn't load libraries</Alert.Title>
      <Alert.Description>{error}</Alert.Description>
      <Alert.Action>
        <Button variant="outline" size="sm" onclick={load}>
          <RotateCwIcon data-icon="inline-start" />
          Retry
        </Button>
      </Alert.Action>
    </Alert.Root>
  {:else}
    <div class="flex flex-col gap-8">
      <section aria-labelledby="libraries-heading" class="flex flex-col gap-3">
        <h2 id="libraries-heading" class="flex items-center gap-3 text-base font-medium">
          <FolderOpenIcon class="size-4 text-muted-foreground" />
          Configured libraries
        </h2>
        {#if status === "loading"}
          <div class="divide-y rounded-xl border" aria-busy="true">
            {#each [0, 1] as i (i)}
              <div class="flex items-center gap-4 px-5 py-4">
                <div class="flex flex-1 flex-col gap-2">
                  <Skeleton class="h-4 w-40" />
                  <Skeleton class="h-3 w-64" />
                </div>
                <Skeleton class="h-8 w-20" />
              </div>
            {/each}
          </div>
        {:else if libraries.length === 0}
          <p class="rounded-xl border px-5 py-8 text-center text-sm text-muted-foreground">
            No libraries yet. Add a folder below, then scan it.
          </p>
        {:else}
          <ul class="divide-y rounded-xl border">
            {#each libraries as library (library.id)}
              <LibraryRow {library} />
            {/each}
          </ul>
        {/if}
      </section>

      <SettingsSection
        icon={FolderPlusIcon}
        title="Add a library"
        description="Use a folder path as the server sees it, for example a mounted media drive."
      >
        <AddLibraryForm oncreated={(library) => (libraries = [...libraries, library])} />
      </SettingsSection>
    </div>
  {/if}
</Page>
