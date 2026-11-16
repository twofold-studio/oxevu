<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import NumberField from "$lib/features/settings/playback/NumberField.svelte";
  import SwitchField from "$lib/features/settings/playback/SwitchField.svelte";
  import { Autosave } from "$lib/features/settings/autosave";
  import type { ProfileRecord, AcquisitionProfile } from "$lib/api/acquisition-types";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { errorMessage } from "$lib/api/client";
  let { open = $bindable(), record = null, onsaved }: { open: boolean; record?: ProfileRecord | null; onsaved: () => void } = $props();
  const defaults: AcquisitionProfile = { name: "1080p", min_height: 720, max_height: 1080, min_size_mb: 0, max_size_mb: 20480, min_seeders: 1, codecs: ["h264", "hevc", "av1"], sources: [], languages: [], allow_hdr: false };
  // svelte-ignore state_referenced_locally
  let draft = $state(structuredClone($state.snapshot(record?.profile ?? defaults)));
  let error = $state<string | null>(null);let status = $state("Changes save automatically.");let pending = $state(false);
  const valid = $derived(Boolean(draft.name.trim() && [draft.min_height,draft.max_height,draft.min_size_mb,draft.max_size_mb,draft.min_seeders].every((n) => Number.isInteger(n) && n >= 0) && draft.min_height <= draft.max_height && draft.min_size_mb <= draft.max_size_mb && draft.max_height <= 4320 && draft.max_size_mb <= 1000000));
  const numbers = [{ key: "min_height", label: "Minimum resolution", unit: "p", max: 4320 },{ key: "max_height", label: "Maximum resolution", unit: "p", max: 4320 },{ key: "min_size_mb", label: "Minimum release size", unit: "MiB", max: 1000000 },{ key: "max_size_mb", label: "Maximum release size", unit: "MiB", max: 1000000 },{ key: "min_seeders", label: "Minimum seeders", unit: undefined, max: 100000 }] as const;
  const autosave = new Autosave<AcquisitionProfile>(async (value) => { if (record) { await acquisitionApi.profile(record.id, value);onsaved(); } }, (state, failure) => { error = state === "error" ? errorMessage(failure) : null;status = state === "saving" ? "Saving changes…" : state === "error" ? "Changes were not saved." : "Changes saved."; });
  untrack(() => autosave.reset($state.snapshot(draft)));
  $effect(() => { const value = record && valid ? $state.snapshot(draft) : null;untrack(() => autosave.update(value)); });
  onDestroy(() => { void autosave.flush(); });
  function values(value: string) { return value.split(",").map((s) => s.trim().toLowerCase()).filter(Boolean); }
  async function create(event: SubmitEvent) { event.preventDefault();if (!valid || record) return;pending = true;error = null;try { await acquisitionApi.profile(crypto.randomUUID(), $state.snapshot(draft));onsaved();open = false; } catch (e) { error = errorMessage(e); } finally { pending = false; } }
</script>
<Dialog.Root bind:open><Dialog.Content><Dialog.Header><Dialog.Title>{record ? "Edit release profile" : "Add release profile"}</Dialog.Title><Dialog.Description>These preferences select downloads. Playback conversion quality is configured separately.</Dialog.Description></Dialog.Header>
  <form onsubmit={create}><Field.Group>
    <Field.Field><Field.Label for="profile-name">Name</Field.Label><Input id="profile-name" bind:value={draft.name} maxlength={100} /></Field.Field>
    <div class="grid gap-4 sm:grid-cols-2">{#each numbers as field (field.key)}<NumberField id={`profile-${field.key}`} label={field.label} min={0} max={field.max} unit={field.unit} bind:value={draft[field.key]} />{/each}</div>
    <Field.Field><Field.Label for="profile-codecs">Allowed codecs</Field.Label><Input id="profile-codecs" value={draft.codecs.join(", ")} oninput={(e) => draft.codecs = values(e.currentTarget.value)} placeholder="h264, hevc, av1" /><Field.Description>Comma-separated. Empty allows any codec, including unknown.</Field.Description></Field.Field>
    <Field.Field><Field.Label for="profile-sources">Allowed sources</Field.Label><Input id="profile-sources" value={draft.sources.join(", ")} oninput={(e) => draft.sources = values(e.currentTarget.value)} placeholder="webdl, webrip, bluray, hdtv" /><Field.Description>Empty allows any source.</Field.Description></Field.Field>
    <Field.Field><Field.Label for="profile-languages">Allowed languages</Field.Label><Input id="profile-languages" value={draft.languages.join(", ")} oninput={(e) => draft.languages = values(e.currentTarget.value)} placeholder="en, bg" /><Field.Description>Filename language detection is limited. Unknown languages require manual review when this list is set.</Field.Description></Field.Field>
    <SwitchField id="profile-hdr" label="Allow HDR releases" description="Playback still depends on the device; HDR conversion is currently unsupported." bind:checked={draft.allow_hdr} />
    {#if !valid}<p class="text-sm text-muted-foreground">Check the profile name and numeric ranges.</p>{/if}
    {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
    <Dialog.Footer>{#if record}<p role="status" class="text-sm text-muted-foreground">{status}</p>{:else}<Button variant="outline" onclick={() => open = false}>Cancel</Button><Button type="submit" disabled={!valid || pending}>{pending ? "Adding…" : "Add profile"}</Button>{/if}</Dialog.Footer>
  </Field.Group></form>
</Dialog.Content></Dialog.Root>
