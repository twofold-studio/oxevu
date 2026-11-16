<script lang="ts">
  import { onMount } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import * as Select from "$lib/components/ui/select";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import SwitchField from "$lib/features/settings/playback/SwitchField.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { errorMessage } from "$lib/api/client";
  let { open = $bindable(), onadded }: { open: boolean; onadded: () => void } = $props();
  let magnet = $state("");let encoded = $state<string | null>(null);let filename = $state("");
  let paused = $state(false);
  let pending = $state(false);let error = $state<string | null>(null);let title = $state("none");let titles = $state<{ value: string; label: string }[]>([]);
  onMount(async () => { try { titles = (await acquisitionApi.monitored()).titles.map((e) => ({ value: e.title.id, label: e.title.title })); } catch { /* Download-only managers may not view the wanted catalog. */ } });
  async function choose(event: Event) {
    const file = (event.target as HTMLInputElement).files?.[0];error = null;encoded = null;filename = "";
    if (!file) return;
    if (file.size > 2 * 1024 * 1024) { error = "Torrent files must be under 2 MiB."; return; }
    const bytes = new Uint8Array(await file.arrayBuffer());let binary = "";
    for (let offset = 0; offset < bytes.length; offset += 32768) binary += String.fromCharCode(...bytes.subarray(offset, offset + 32768));
    encoded = btoa(binary);filename = file.name;magnet = "";
  }
  async function submit(event: SubmitEvent) {
    event.preventDefault();pending = true;error = null;
    try { await acquisitionApi.download({ paused, ...(encoded ? { torrent_base64: encoded } : { magnet: magnet.trim() }), ...(title !== "none" ? { title_id: title } : {}) }); open = false; onadded(); }
    catch (e) { error = errorMessage(e); } finally { pending = false; }
  }
</script>
<Dialog.Root bind:open><Dialog.Content>
  <Dialog.Header><Dialog.Title>Add a download</Dialog.Title><Dialog.Description>Use a magnet link or torrent file. Direct traffic is permitted. Removing a queue entry retains its files.</Dialog.Description></Dialog.Header>
  <form onsubmit={submit}><Field.Group>
    <Field.Field><Field.Label for="torrent-magnet">Magnet link</Field.Label><Input id="torrent-magnet" bind:value={magnet} maxlength={16384} placeholder="magnet:?xt=urn:btih:…" oninput={() => { encoded = null; filename = ""; }} /></Field.Field>
    <Field.Field><Field.Label for="torrent-file">Or a torrent file</Field.Label><Input id="torrent-file" type="file" accept=".torrent,application/x-bittorrent" onchange={choose} /><Field.Description>{filename || "Up to 2 MiB. Pure v2 torrents are not supported."}</Field.Description></Field.Field>
    <SwitchField id="torrent-paused" label="Start paused" description="Use this when migrating an existing torrent. Adopt its data before explicitly resuming." bind:checked={paused} />
    {#if titles.length}<Field.Field><Field.Label for="torrent-title">Associate with a wanted title</Field.Label><Select.Root type="single" items={[{ value: "none", label: "Standalone transfer" }, ...titles]} bind:value={title}><Select.Trigger id="torrent-title" class="w-full"><Select.Value /></Select.Trigger><Select.Content><Select.Group><Select.Item value="none">Standalone transfer</Select.Item>{#each titles as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content></Select.Root><Field.Description>Imports need a catalog identity and a configured destination.</Field.Description></Field.Field>{/if}
    {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
    <Dialog.Footer><Button variant="outline" onclick={() => open = false}>Cancel</Button><Button type="submit" disabled={pending || (!encoded && !magnet.trim())}>{pending ? "Queuing…" : "Add download"}</Button></Dialog.Footer>
  </Field.Group></form>
</Dialog.Content></Dialog.Root>
