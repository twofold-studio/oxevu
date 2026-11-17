<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import SwitchField from "$lib/features/settings/playback/SwitchField.svelte";
  import { Autosave } from "$lib/features/settings/autosave";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { Indexer } from "$lib/api/acquisition-types";
  import { errorMessage } from "$lib/api/client";
  let { open = $bindable(), indexer = null, onsaved }: { open: boolean; indexer?: Indexer | null; onsaved: () => void } = $props();
  // svelte-ignore state_referenced_locally
  let draft = $state({ name: indexer?.name ?? "", endpoint: indexer?.endpoint ?? "", enabled: indexer?.enabled ?? false, api_key: "", hosts: indexer?.download_hosts.join(", ") ?? "" });
  let error = $state<string | null>(null);let pending = $state(false);let status = $state("Changes save automatically.");
  const body = $derived({ name: draft.name.trim(), endpoint: draft.endpoint.trim(), enabled: draft.enabled, api_key: draft.api_key, download_hosts: draft.hosts.split(",").map((s) => s.trim()).filter(Boolean) });
  const valid = $derived(Boolean(body.name && /^https?:\/\//.test(body.endpoint)));
  const autosave = new Autosave<typeof body>(async (value) => { if (indexer) { await acquisitionApi.indexer(indexer.id, value);onsaved(); } }, (state, failure) => { error = state === "error" ? errorMessage(failure) : null;status = state === "saving" ? "Saving changes…" : state === "error" ? "Changes were not saved." : "Changes saved."; });
  untrack(() => autosave.reset($state.snapshot(body)));
  $effect(() => { const value = indexer && valid ? $state.snapshot(body) : null;untrack(() => autosave.update(value)); });
  onDestroy(() => { void autosave.flush(); });
  async function create(event: SubmitEvent) { event.preventDefault();if (!valid || indexer) return;pending = true;error = null;try { await acquisitionApi.indexer(crypto.randomUUID(), $state.snapshot(body));onsaved();open = false; } catch (e) { error = errorMessage(e); } finally { pending = false; } }
</script>
<Dialog.Root bind:open><Dialog.Content><Dialog.Header><Dialog.Title>{indexer ? "Edit indexer" : "Add indexer"}</Dialog.Title><Dialog.Description>Use a supported direct source or a Torznab endpoint. Credentials stay on the server.</Dialog.Description></Dialog.Header>
  <form onsubmit={create}><Field.Group>
    <Field.Field><Field.Label for="indexer-name">Name</Field.Label><Input id="indexer-name" bind:value={draft.name} maxlength={100} placeholder="Home indexer" /></Field.Field>
    <Field.Field><Field.Label for="indexer-url">API endpoint</Field.Label><Input id="indexer-url" type="url" bind:value={draft.endpoint} maxlength={4096} placeholder="https://api.knaben.org/v1" /><Field.Description>Use the full endpoint without an API-key query parameter.</Field.Description></Field.Field>
    <Field.Field><Field.Label for="indexer-key">API key</Field.Label><Input id="indexer-key" type="password" bind:value={draft.api_key} autocomplete="new-password" placeholder={indexer?.api_key_set ? "Stored · leave blank to retain" : "Optional if the source is public"} /></Field.Field>
    <Field.Field><Field.Label for="indexer-hosts">Additional torrent hosts</Field.Label><Input id="indexer-hosts" bind:value={draft.hosts} placeholder="downloads.example.org, tracker.example.org" /><Field.Description>Comma-separated hostnames. External download hosts must use HTTPS; redirects are not followed.</Field.Description></Field.Field>
    <SwitchField id="indexer-enabled" label="Enabled" description="Include this source in release searches." bind:checked={draft.enabled} />
    {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
    <Dialog.Footer>{#if indexer}<p role="status" class="text-sm text-muted-foreground">{status}</p>{:else}<Button variant="outline" onclick={() => open = false}>Cancel</Button><Button type="submit" disabled={pending || !valid}>{pending ? "Adding…" : "Add indexer"}</Button>{/if}</Dialog.Footer>
  </Field.Group></form>
</Dialog.Content></Dialog.Root>
