<script lang="ts">
  import { onMount } from "svelte";
  import FolderInputIcon from "@lucide/svelte/icons/folder-input";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import ScanIcon from "@lucide/svelte/icons/scan";
  import FilmIcon from "@lucide/svelte/icons/film";
  import TvIcon from "@lucide/svelte/icons/tv";
  import * as Field from "$lib/components/ui/field";
  import * as Select from "$lib/components/ui/select";
  import { acquisitionDocument } from "$lib/features/settings/documents";
  import { createSettingsForm } from "$lib/features/settings/settings-form.svelte";
  import SettingsFrame from "$lib/features/settings/layout/SettingsFrame.svelte";
  import SettingsFormBody from "$lib/features/settings/layout/SettingsFormBody.svelte";
  import SettingsRow from "$lib/features/settings/layout/SettingsRow.svelte";
  import SwitchField from "$lib/features/settings/playback/SwitchField.svelte";
  import { validateAcquisition } from "./acquisition-form";
  import { librariesApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import type { Library } from "$lib/api/types";
  import type { AcquisitionConfig, ImportDestination, ImportDefaults } from "$lib/api/acquisition-types";
  import { acquisitionApi } from "$lib/api/acquisition";

  const form = createSettingsForm(acquisitionDocument, ["import_enabled", "copy_fallback", "auto_identify"], validateAcquisition);

  let libraries = $state<Library[]>([]);
  let destError = $state<string | null>(null);
  let destinationSaving = $state(false);
  let defaults = $state<ImportDefaults | null>(null);

  onMount(async () => {
    try {
      const [listed, effective] = await Promise.all([librariesApi.list(), acquisitionApi.importDefaults()]);
      libraries = listed.libraries;
      defaults = effective;
    }
    catch (error) { destError = errorMessage(error); }
  });

  const movieDest = $derived(form.draft?.movie_destination ?? null);
  const seriesDest = $derived(form.draft?.series_destination ?? null);

  const movieLibraryOptions = $derived([
    { value: "auto", label: "Automatic" },
    ...libraries.filter((l) => l.enabled && l.kind === "movies").map((l) => ({ value: l.id, label: l.name })),
  ]);
  const movieRootOptions = $derived(
    (libraries.find((l) => l.id === movieDest?.library_id)?.paths ?? []).map((p) => ({ value: p, label: p })),
  );
  const seriesLibraryOptions = $derived([
    { value: "auto", label: "Automatic" },
    ...libraries.filter((l) => l.enabled && l.kind === "tv").map((l) => ({ value: l.id, label: l.name })),
  ]);
  const seriesRootOptions = $derived(
    (libraries.find((l) => l.id === seriesDest?.library_id)?.paths ?? []).map((p) => ({ value: p, label: p })),
  );

  async function saveDestination(key: "movie_destination" | "series_destination", value: ImportDestination | null) {
    if (destinationSaving) return;
    destinationSaving = true;
    destError = null;
    try {
      await acquisitionDocument.patch({ [key]: value } as Partial<AcquisitionConfig>);
      if (form.draft) form.draft[key] = value;
      defaults = await acquisitionApi.importDefaults();
    } catch (e) {
      destError = errorMessage(e);
    } finally {
      destinationSaving = false;
    }
  }

  function chooseMovieLibrary(value: string) {
    if (destinationSaving) return;
    if (value === "auto") {
      void saveDestination("movie_destination", null);
      return;
    }
    const paths = libraries.find((l) => l.id === value)?.paths ?? [];
    const root = paths.length === 1 ? paths[0] : "";
    if (form.draft) form.draft.movie_destination = { library_id: value, root };
    if (root) void saveDestination("movie_destination", { library_id: value, root });
  }

  function chooseMovieRoot(value: string) {
    if (!movieDest) return;
    void saveDestination("movie_destination", { library_id: movieDest.library_id, root: value, ...(movieDest.root_device ? { root_device: movieDest.root_device } : {}) });
  }

  function chooseSeriesLibrary(value: string) {
    if (destinationSaving) return;
    if (value === "auto") {
      void saveDestination("series_destination", null);
      return;
    }
    const paths = libraries.find((l) => l.id === value)?.paths ?? [];
    const root = paths.length === 1 ? paths[0] : "";
    if (form.draft) form.draft.series_destination = { library_id: value, root };
    if (root) void saveDestination("series_destination", { library_id: value, root });
  }

  function chooseSeriesRoot(value: string) {
    if (!seriesDest) return;
    void saveDestination("series_destination", { library_id: seriesDest.library_id, root: value, ...(seriesDest.root_device ? { root_device: seriesDest.root_device } : {}) });
  }
</script>

<SettingsFrame
  page="downloads/imports"
  description="Whether completed downloads may be written into libraries. Library folders stay read-only unless this is on."
>
  <SettingsFormBody {form} what="download settings">
    {#if form.draft}
      <SettingsRow
        icon={FolderInputIcon}
        title="Library writes"
        description="Completed, verified media imports into the default movie or series folder. Per-title destinations override the defaults."
      >
        <SwitchField
          id="acq-imports"
          label="Automatically import completed media"
          description="Allow writes into configured library destinations and reviewed manual imports. Ambiguous files need review; the seeding source is retained."
          bind:checked={form.draft.import_enabled}
        />
      </SettingsRow>
      <SettingsRow
        icon={CopyIcon}
        title="Across filesystems"
        description="Imports hardlink by default so the seeding copy and the library copy share space."
      >
        <SwitchField
          id="acq-copy"
          label="Cross-filesystem copy fallback"
          description="If a hardlink is impossible across filesystems, copy and verify the file. The seeding source remains."
          bind:checked={form.draft.copy_fallback}
        />
      </SettingsRow>
      <SettingsRow
        icon={ScanIcon}
        title="Content recognition"
        description="Downloads without an explicit title association are matched against the catalog by name."
      >
        <SwitchField
          id="acq-identify"
          label="Identify downloads automatically"
          description="When a download has no title association, attempt to match it to a known movie or series from its name. Unrecognized downloads can be identified manually."
          bind:checked={() => form.draft?.auto_identify ?? true, (value) => { if (form.draft) form.draft.auto_identify = value; }}
        />
      </SettingsRow>
    {/if}
  </SettingsFormBody>

  {#if form.draft}
    <div class="mt-6 flex flex-col gap-4">
      <h2 class="text-base font-semibold tracking-tight">Default import destinations</h2>
      <p class="max-w-2xl text-sm text-muted-foreground">Choose a default library and folder for each type. Per-title monitor destinations override these defaults. Automatic uses the first enabled library of each type.</p>
      {#if destError}<p role="alert" class="text-sm text-destructive">{destError}</p>{/if}

      <div class="divide-y rounded-xl border bg-card">
        <section class="grid gap-5 p-5 md:grid-cols-[minmax(12rem,0.72fr)_minmax(0,1.5fr)] md:gap-10 md:p-6">
          <div class="flex gap-3">
            <FilmIcon class="mt-1 size-4 shrink-0 text-muted-foreground" />
            <div class="flex flex-col gap-1">
              <h2 class="text-base leading-snug font-medium">Movies</h2>
              <p class="text-sm leading-6 text-muted-foreground">Default destination for movie imports.</p>
            </div>
          </div>
          <div class="max-w-2xl min-w-0">
            <Field.Group>
              <Field.Field>
                <Field.Label for="dest-movie-lib">Library</Field.Label>
                <Select.Root type="single" disabled={destinationSaving} items={movieLibraryOptions} value={movieDest?.library_id ?? "auto"} onValueChange={chooseMovieLibrary}>
                  <Select.Trigger id="dest-movie-lib" class="w-full"><Select.Value /></Select.Trigger>
                  <Select.Content><Select.Group>{#each movieLibraryOptions as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content>
                </Select.Root>
              </Field.Field>
              {#if movieDest?.library_id}
                <Field.Field>
                  <Field.Label for="dest-movie-root">Folder</Field.Label>
                  <Select.Root type="single" disabled={destinationSaving} items={movieRootOptions} value={movieDest.root || ""} onValueChange={chooseMovieRoot}>
                    <Select.Trigger id="dest-movie-root" class="w-full"><Select.Value placeholder="Choose folder" /></Select.Trigger>
                    <Select.Content><Select.Group>{#each movieRootOptions as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content>
                  </Select.Root>
                </Field.Field>
              {/if}
              {#if !movieDest}<p class="break-all text-xs text-muted-foreground">Default: {defaults?.movies?.root ?? "Add an enabled Movies library to choose a default."}</p>{/if}
            </Field.Group>
          </div>
        </section>

        <section class="grid gap-5 p-5 md:grid-cols-[minmax(12rem,0.72fr)_minmax(0,1.5fr)] md:gap-10 md:p-6">
          <div class="flex gap-3">
            <TvIcon class="mt-1 size-4 shrink-0 text-muted-foreground" />
            <div class="flex flex-col gap-1">
              <h2 class="text-base leading-snug font-medium">Series</h2>
              <p class="text-sm leading-6 text-muted-foreground">Default destination for series imports.</p>
            </div>
          </div>
          <div class="max-w-2xl min-w-0">
            <Field.Group>
              <Field.Field>
                <Field.Label for="dest-series-lib">Library</Field.Label>
                <Select.Root type="single" disabled={destinationSaving} items={seriesLibraryOptions} value={seriesDest?.library_id ?? "auto"} onValueChange={chooseSeriesLibrary}>
                  <Select.Trigger id="dest-series-lib" class="w-full"><Select.Value /></Select.Trigger>
                  <Select.Content><Select.Group>{#each seriesLibraryOptions as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content>
                </Select.Root>
              </Field.Field>
              {#if seriesDest?.library_id}
                <Field.Field>
                  <Field.Label for="dest-series-root">Folder</Field.Label>
                  <Select.Root type="single" disabled={destinationSaving} items={seriesRootOptions} value={seriesDest.root || ""} onValueChange={chooseSeriesRoot}>
                    <Select.Trigger id="dest-series-root" class="w-full"><Select.Value placeholder="Choose folder" /></Select.Trigger>
                    <Select.Content><Select.Group>{#each seriesRootOptions as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content>
                  </Select.Root>
                </Field.Field>
              {/if}
              {#if !seriesDest}<p class="break-all text-xs text-muted-foreground">Default: {defaults?.tv?.root ?? "Add an enabled Series library to choose a default."}</p>{/if}
            </Field.Group>
          </div>
        </section>
      </div>
    </div>
  {/if}
</SettingsFrame>
