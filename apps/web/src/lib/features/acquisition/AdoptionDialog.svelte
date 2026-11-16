<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Button } from "$lib/components/ui/button";
  import PathInput from "$lib/features/settings/libraries/PathInput.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { Download, ImportFile } from "$lib/api/acquisition-types";
  import { errorMessage } from "$lib/api/client";
  import { bytes } from "./format";
  let { open = $bindable(), job, onadopted }: { open: boolean; job: Download; onadopted: () => void } = $props();
  let root = $state("");let preview = $state<{ files: ImportFile[]; token: string; notice: string } | null>(null);let reviewedRoot = $state("");
  let pending = $state(false);let error = $state<string | null>(null);
  async function review() { pending = true;error = null;try { preview = await acquisitionApi.previewAdoption(job.id, root);reviewedRoot = root; } catch (e) { error = errorMessage(e); } finally { pending = false; } }
  async function adopt() { if (!preview || root !== reviewedRoot) return;pending = true;error = null;try { await acquisitionApi.adopt(job.id, root, preview.token);open = false;onadopted(); } catch (e) { error = errorMessage(e); } finally { pending = false; } }
</script>
<Dialog.Root bind:open><Dialog.Content size="wide"><Dialog.Header><Dialog.Title>Adopt existing torrent data</Dialog.Title><Dialog.Description>Pause the old client's transfer first. Choose the folder containing the torrent's files. Oxevu copies existing data into its own storage; it never changes originals or trusts another client's fast-resume state.</Dialog.Description></Dialog.Header>
  <Field.Field><Field.Label>Existing download folder</Field.Label><PathInput bind:value={root} label="Existing download folder" /></Field.Field>
  <Button variant="outline" disabled={pending || !root.startsWith("/")} onclick={review}>{pending ? "Working…" : "Preview copies"}</Button>
  {#if preview && root === reviewedRoot}<p class="text-sm text-muted-foreground">{preview.notice}</p><ul class="flex flex-col gap-3">{#each preview.files as file (file.file_index)}<li class="text-sm break-all"><p>{file.source}</p><p class="text-muted-foreground">→ {file.destination} · {bytes(file.size)}</p></li>{/each}</ul>{/if}
  {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
  <Dialog.Footer><Button variant="outline" disabled={pending} onclick={() => open = false}>Close</Button><Button disabled={pending || !preview || root !== reviewedRoot} onclick={adopt}>{pending ? "Copying and verifying…" : "Copy reviewed data"}</Button></Dialog.Footer>
</Dialog.Content></Dialog.Root>
