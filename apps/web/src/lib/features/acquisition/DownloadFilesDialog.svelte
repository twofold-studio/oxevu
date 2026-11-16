<script lang="ts">
  import { onDestroy } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import type { Download } from "$lib/api/acquisition-types";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { errorMessage } from "$lib/api/client";
  import { Autosave } from "$lib/features/settings/autosave";
  import { session } from "$lib/state/session.svelte";
  import { bytes } from "./format";
  import AdoptionDialog from "./AdoptionDialog.svelte";
  let { open = $bindable(), job, onsaved, onimport }: { open: boolean; job: Download; onsaved: () => void; onimport: () => void } = $props();
  let selected = $state<number[] | null>(null);let error = $state<string | null>(null);let status = $state("Selection changes save automatically and pause the transfer.");
  let adoptionOpen = $state(false);
  const selection = $derived(selected ?? job.selected_files ?? job.files.map((f) => f.index));
  const autosave = new Autosave<number[]>(async (files) => { await acquisitionApi.control(job.id, "select", files); onsaved(); }, (state, e) => { error = state === "error" ? errorMessage(e) : null; status = state === "saving" ? "Applying selection…" : state === "error" ? "Selection was not applied." : "Selection applied. Resume the transfer from Activity when ready."; });
  function toggle(index: number, checked: boolean) { selected = checked ? [...selection, index] : selection.filter((i) => i !== index); if (selected.length) autosave.update($state.snapshot(selected)); else error = "Select at least one file."; }
  onDestroy(() => { void autosave.flush(); });
</script>
{#if !adoptionOpen}<Dialog.Root bind:open><Dialog.Content size="wide">
  <Dialog.Header><Dialog.Title>{job.title?.title ?? "Torrent files"}</Dialog.Title><Dialog.Description>Choose files to transfer. Unselected files are retained if already downloaded.</Dialog.Description></Dialog.Header>
  {#if job.infohash}<p class="text-xs break-all text-muted-foreground">Infohash · {job.infohash}</p>{/if}
  {#if !job.files.length}<p class="text-sm text-muted-foreground">Waiting for torrent metadata. This list updates as the job is initialized.</p>{:else}
    <Field.Set><Field.Legend variant="label">Files</Field.Legend><Field.Group>
      {#each job.files as file (file.index)}
        <Field.Field orientation="horizontal"><Checkbox id={`torrent-file-${file.index}`} checked={selection.includes(file.index)} onCheckedChange={(value) => toggle(file.index, value === true)} /><Field.Content><Field.Label for={`torrent-file-${file.index}`} class="break-all">{file.path}</Field.Label><Field.Description>{bytes(job.stats.file_progress?.[file.index] ?? 0)} / {bytes(file.size)}</Field.Description></Field.Content></Field.Field>
      {/each}
    </Field.Group></Field.Set>
  {/if}
  {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
  <p role="status" class="text-sm text-muted-foreground">{status}</p>
  <Dialog.Footer>{#if session.user?.is_admin && job.state === "paused"}<Button variant="outline" onclick={() => adoptionOpen = true}>Adopt existing data</Button>{/if}{#if job.title && session.user?.is_admin && job.files.length}<Button variant="outline" onclick={onimport}>Review import</Button>{/if}<Button onclick={() => open = false}>Done</Button></Dialog.Footer>
</Dialog.Content></Dialog.Root>{/if}
{#if adoptionOpen}<AdoptionDialog bind:open={adoptionOpen} {job} onadopted={onsaved} />{/if}
