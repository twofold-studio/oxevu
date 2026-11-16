<script lang="ts">
  import { onMount, onDestroy, untrack } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import * as Select from "$lib/components/ui/select";
  import type { MonitoredTitle, Monitor, CatalogEpisode, ProfileRecord } from "$lib/api/acquisition-types";
  import type { Library } from "$lib/api/types";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { librariesApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import SwitchField from "$lib/features/settings/playback/SwitchField.svelte";
  import { Autosave } from "$lib/features/settings/autosave";
  import RequestScope from "./RequestScope.svelte";
  let { open = $bindable(), entry, onsaved }: { open: boolean; entry: MonitoredTitle; onsaved: () => void } = $props();
  // svelte-ignore state_referenced_locally
  let draft = $state<Monitor>(structuredClone($state.snapshot(entry.monitor)));
  let libraries = $state<Library[]>([]);let profiles = $state<ProfileRecord[]>([]);let episodes = $state<CatalogEpisode[]>([]);
  let loading = $state(true);let error = $state<string | null>(null);let status = $state("Changes save automatically.");
  const libraryOptions = $derived([{ value: "none", label: "No destination yet" }, ...libraries.filter((l) => l.enabled && l.kind === (entry.title.kind === "tv" ? "tv" : "movies")).map((l) => ({ value: l.id, label: l.name }))]);
  const rootOptions = $derived((libraries.find((l) => l.id === draft.library_id)?.paths ?? []).map((path) => ({ value: path, label: path })));
  const profileOptions = $derived(profiles.map((p) => ({ value: p.id, label: p.profile.name })));
  const valid = $derived(!loading && Boolean(draft.library_id) === Boolean(draft.root) && (!draft.automatic || Boolean(draft.library_id && draft.root)) && (draft.scope.mode !== "seasons" || draft.scope.seasons.length > 0) && (draft.scope.mode !== "episodes" || draft.scope.episodes.length > 0));
  function chooseLibrary(value: string) {
    draft.library_id = value === "none" ? null : value;
    const paths = libraries.find((library) => library.id === draft.library_id)?.paths ?? [];
    draft.root = paths.length === 1 ? paths[0] : null;
  }
  const autosave = new Autosave<Monitor>(async (value) => { await acquisitionApi.monitor(entry.title.id, value); onsaved(); }, (state, failure) => { error = state === "error" ? errorMessage(failure) : null; status = state === "saving" ? "Saving changes…" : state === "error" ? "Changes were not saved." : "Changes saved."; });
  untrack(() => autosave.reset($state.snapshot(draft)));
  onMount(async () => {
    try { const [l,p,t] = await Promise.all([librariesApi.list(), acquisitionApi.profiles(), acquisitionApi.title(entry.title.kind, entry.title.provider_id)]); libraries = l.libraries; profiles = p.profiles; episodes = t.episodes; }
    catch (e) { error = errorMessage(e); } finally { loading = false; }
  });
  $effect(() => { const value = valid ? $state.snapshot(draft) : null; untrack(() => autosave.update(value)); });
  onDestroy(() => { void autosave.flush(); });
</script>
<Dialog.Root bind:open><Dialog.Content>
  <Dialog.Header><Dialog.Title>Manage {entry.title.title}</Dialog.Title><Dialog.Description>Set monitoring, destination and release preferences. Imports require the separate server-wide import switch.</Dialog.Description></Dialog.Header>
  {#if loading}<p class="text-sm text-muted-foreground">Loading catalog and destinations…</p>{:else}
    <Field.Group>
      <SwitchField id="monitor-enabled" label="Monitoring enabled" description="Pause automatic searches and imports for this title when disabled." bind:checked={draft.enabled} />
      <RequestScope bind:scope={draft.scope} title={entry.title} {episodes} monitoring />
      <Field.Field><Field.Label for="monitor-library">Destination library</Field.Label><Select.Root type="single" items={libraryOptions} value={draft.library_id ?? "none"} onValueChange={chooseLibrary}><Select.Trigger id="monitor-library" class="w-full"><Select.Value /></Select.Trigger><Select.Content><Select.Group>{#each libraryOptions as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content></Select.Root></Field.Field>
      {#if draft.library_id}<Field.Field><Field.Label for="monitor-root">Library folder</Field.Label><Select.Root type="single" items={rootOptions} value={draft.root ?? ""} onValueChange={(v) => draft.root = v || null}><Select.Trigger id="monitor-root" class="w-full"><Select.Value placeholder="Choose a folder" /></Select.Trigger><Select.Content><Select.Group>{#each rootOptions as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content></Select.Root><Field.Description>Only configured folders can receive imports. Existing files are never overwritten.</Field.Description></Field.Field>{/if}
      <Field.Field><Field.Label for="monitor-profile">Release profile</Field.Label><Select.Root type="single" items={profileOptions} bind:value={draft.profile_id}><Select.Trigger id="monitor-profile" class="w-full"><Select.Value /></Select.Trigger><Select.Content><Select.Group>{#each profileOptions as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content></Select.Root></Field.Field>
      <SwitchField id="monitor-automatic" label="Automatic acquisition" description="Search for missing, aired titles and queue releases that match the profile. No automatic upgrades or deletion." bind:checked={draft.automatic} />
      {#if !valid}<p class="text-sm text-muted-foreground">Choose a complete destination and a nonempty episode selection to save.</p>{/if}
      {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
    </Field.Group>
    <Dialog.Footer><p role="status" class="text-sm text-muted-foreground">{status}</p></Dialog.Footer>
  {/if}
</Dialog.Content></Dialog.Root>
