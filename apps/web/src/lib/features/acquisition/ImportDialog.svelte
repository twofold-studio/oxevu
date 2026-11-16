<script lang="ts">
  import { onMount } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import * as Select from "$lib/components/ui/select";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import type { Download, ImportInput, ImportFile } from "$lib/api/acquisition-types";
  import type { Library } from "$lib/api/types";
  import { librariesApi } from "$lib/api/endpoints";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { errorMessage } from "$lib/api/client";
  import { bytes } from "./format";
  let { open = $bindable(), job, onimported }: { open: boolean; job: Download; onimported: () => void } = $props();
  let libraries = $state<Library[]>([]);let libraryId = $state("");let root = $state("");
  let mappings = $state<Record<number, { selected: boolean; season: number; first: number; last: number }>>({});
  let pending = $state(false);let error = $state<string | null>(null);let reviewed = $state<{ files: ImportFile[]; token: string; copy_fallback: boolean } | null>(null);let previewInput = $state("");
  const tv = $derived(job.title?.kind === "tv");
  const files = $derived(job.files.filter((f) => /\.(mkv|mp4|webm|mov|avi|m4v|ts)$/i.test(f.path)));
  const libraryOptions = $derived(libraries.filter((l) => l.enabled && l.kind === (tv ? "tv" : "movies")).map((l) => ({ value: l.id, label: l.name })));
  const rootOptions = $derived((libraries.find((l) => l.id === libraryId)?.paths ?? []).map((path) => ({ value: path, label: path })));
  const body = $derived.by<ImportInput>(() => ({ library_id: libraryId, root, mappings: Object.entries(mappings).filter(([,m]) => m.selected).map(([index,m]) => ({ file_index: Number(index), episodes: tv ? Array.from({ length: Math.max(0, Math.min(20, m.last - m.first + 1)) }, (_,i) => ({ season: m.season, episode: m.first + i })) : [] })) }));
  const valid = $derived(Boolean(libraryId && root && body.mappings.length && (!tv || body.mappings.every((m) => m.episodes.length && m.episodes.every((e) => Number.isInteger(e.season) && Number.isInteger(e.episode) && e.season >= 0 && e.episode >= 0)))));
  const currentPreview = $derived(reviewed && previewInput === JSON.stringify(body) ? reviewed : null);
  onMount(async () => {
    try { libraries = (await librariesApi.list()).libraries; for (const file of files) { const code = /s(\d{1,3})e(\d{1,4})(?:-?e(\d{1,4}))?/i.exec(file.path); mappings[file.index] = { selected: false, season: Number(code?.[1] ?? 1), first: Number(code?.[2] ?? 1), last: Number(code?.[3] ?? code?.[2] ?? 1) }; } }
    catch (e) { error = errorMessage(e); }
  });
  function toggle(index: number, checked: boolean) { if (!tv && checked) for (const m of Object.values(mappings)) m.selected = false; mappings[index].selected = checked; }
  async function preview() { pending = true;error = null;try { const snapshot = $state.snapshot(body); reviewed = await acquisitionApi.previewImport(job.id, snapshot); previewInput = JSON.stringify(snapshot); } catch (e) { error = errorMessage(e); } finally { pending = false; } }
  async function publish() { if (!currentPreview) return;pending = true;error = null;try { await acquisitionApi.import(job.id, { ...$state.snapshot(body), token: currentPreview.token }); onimported(); open = false; } catch (e) { error = errorMessage(e); } finally { pending = false; } }
</script>
<Dialog.Root bind:open><Dialog.Content size="wide">
  <Dialog.Header><Dialog.Title>Import {job.title?.title ?? "media"}</Dialog.Title><Dialog.Description>Review file mappings and destinations. Imports retain the download source and refuse to overwrite existing media.</Dialog.Description></Dialog.Header>
  <Field.Group>
    <Field.Field><Field.Label for="import-library">Library</Field.Label><Select.Root type="single" items={libraryOptions} bind:value={() => libraryId, (value) => { libraryId = value; root = ""; }}><Select.Trigger id="import-library" class="w-full"><Select.Value placeholder="Choose library" /></Select.Trigger><Select.Content><Select.Group>{#each libraryOptions as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content></Select.Root></Field.Field>
    <Field.Field><Field.Label for="import-root">Folder</Field.Label><Select.Root type="single" items={rootOptions} bind:value={root}><Select.Trigger id="import-root" class="w-full"><Select.Value placeholder="Choose library folder" /></Select.Trigger><Select.Content><Select.Group>{#each rootOptions as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content></Select.Root></Field.Field>
    <Field.Set><Field.Legend variant="label">Verified video files</Field.Legend><Field.Group>
      {#each files as file (file.index)}
        {#if mappings[file.index]}
          <Field.Field><div class="flex items-start gap-3"><Checkbox id={`import-file-${file.index}`} checked={mappings[file.index].selected} disabled={job.stats.file_progress?.[file.index] !== file.size} onCheckedChange={(value) => toggle(file.index, value === true)} /><Field.Content><Field.Label for={`import-file-${file.index}`} class="break-all">{file.path}</Field.Label><Field.Description>{bytes(file.size)} · {job.stats.file_progress?.[file.index] === file.size ? "Verified" : "Incomplete"}</Field.Description></Field.Content></div>
          {#if tv && mappings[file.index].selected}<div class="grid grid-cols-3 gap-3"><Field.Field><Field.Label for={`map-season-${file.index}`}>Season</Field.Label><Input id={`map-season-${file.index}`} type="number" min={0} max={999} bind:value={mappings[file.index].season} /></Field.Field><Field.Field><Field.Label for={`map-first-${file.index}`}>First episode</Field.Label><Input id={`map-first-${file.index}`} type="number" min={0} max={9999} bind:value={mappings[file.index].first} /></Field.Field><Field.Field><Field.Label for={`map-last-${file.index}`}>Last episode</Field.Label><Input id={`map-last-${file.index}`} type="number" min={0} max={9999} bind:value={mappings[file.index].last} /></Field.Field></div>{/if}</Field.Field>
        {/if}
      {/each}
    </Field.Group></Field.Set>
    {#if currentPreview}<section class="rounded-lg border p-4"><h3 class="mb-3 text-sm font-medium">Destination preview</h3><ul class="flex flex-col gap-3">{#each currentPreview.files as file (file.file_index)}<li class="text-xs break-all text-muted-foreground">{file.destination}</li>{/each}</ul><p class="mt-3 text-xs text-muted-foreground">Hardlinks are preferred. Cross-filesystem copy fallback is {currentPreview.copy_fallback ? "enabled" : "disabled"}.</p></section>{/if}
    {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
  </Field.Group>
  <Dialog.Footer><Button variant="outline" onclick={() => open = false}>Close</Button>{#if currentPreview}<Button disabled={pending} onclick={publish}>{pending ? "Importing…" : "Import verified files"}</Button>{:else}<Button disabled={pending || !valid} onclick={preview}>{pending ? "Checking…" : "Preview destinations"}</Button>{/if}</Dialog.Footer>
</Dialog.Content></Dialog.Root>
