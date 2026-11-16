<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import type { UserSettings } from "$lib/api/admin-types";
  import type { AcquisitionPermissions } from "$lib/api/acquisition-types";
  import * as Field from "$lib/components/ui/field";
  import SwitchField from "$lib/features/settings/playback/SwitchField.svelte";
  import NumberField from "$lib/features/settings/playback/NumberField.svelte";
  import { Autosave } from "$lib/features/settings/autosave";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { errorMessage } from "$lib/api/client";
  let { user }: { user: UserSettings } = $props();
  const defaults: AcquisitionPermissions = { can_request: true, auto_approve: false, manage_requests: false, manage_downloads: false, manage_indexers: false, request_limit: 10 };
  // svelte-ignore state_referenced_locally
  let draft = $state(structuredClone($state.snapshot(user.acquisition ?? defaults)));
  let error = $state<string | null>(null);let status = $state("Changes save automatically.");
  const autosave = new Autosave<AcquisitionPermissions>(async (value) => { await acquisitionApi.permissions(user.id, value); }, (state, failure) => { error = state === "error" ? errorMessage(failure) : null;status = state === "saving" ? "Saving changes…" : state === "error" ? "Changes were not saved." : "Changes saved."; });
  untrack(() => autosave.reset($state.snapshot(draft)));
  $effect(() => { const value = Number.isInteger(draft.request_limit) && draft.request_limit >= 0 && draft.request_limit <= 1000 ? $state.snapshot(draft) : null;untrack(() => autosave.update(value)); });
  onDestroy(() => { void autosave.flush(); });
</script>
{#if user.is_admin}<p class="text-sm text-muted-foreground">Administrators have full acquisition access. These limits apply to non-administrator accounts.</p>{:else}
  <Field.Group>
    <SwitchField id="request-allowed" label="Can request media" description="Request movies, series, seasons and episodes." bind:checked={draft.can_request} />
    <SwitchField id="request-auto" label="Approve this user's requests automatically" description="Bypass request review. Per-title automatic acquisition is still a separate setting." bind:checked={draft.auto_approve} />
    <NumberField id="request-limit" label="Active request limit" description="Pending and approved requests count toward this limit. Zero prevents new requests." min={0} max={1000} bind:value={draft.request_limit} />
    <SwitchField id="request-manage" label="Review household requests" description="Approve, decline and view other users' requests." bind:checked={draft.manage_requests} />
    <SwitchField id="download-manage" label="Manage transfers" description="Search releases, add torrents and control the shared queue." bind:checked={draft.manage_downloads} />
    <SwitchField id="indexer-manage" label="Manage indexers" description="Configure endpoints, API keys and allowed download hosts." bind:checked={draft.manage_indexers} />
    <p role="status" class="text-sm text-muted-foreground">{#if error}<span class="text-destructive">{error}</span>{:else}{status}{/if}</p>
  </Field.Group>
{/if}
