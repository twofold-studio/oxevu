<script lang="ts">
  import { onMount } from "svelte";
  import ShieldCheckIcon from "@lucide/svelte/icons/shield-check";
  import * as Field from "$lib/components/ui/field";
  import * as Select from "$lib/components/ui/select";
  import SettingsSection from "$lib/features/settings/SettingsSection.svelte";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import { usersApi } from "$lib/api/endpoints";
  import PermissionFields from "./PermissionFields.svelte";
  const resource = createResource(async () => (await usersApi.list()).users);
  let selected = $state("");
  const options = $derived((resource.value ?? []).map((u) => ({ value: u.id, label: u.username })));
  const user = $derived(resource.value?.find((u) => u.id === selected));
  onMount(() => { void resource.load(); });
</script>
<SettingsSection icon={ShieldCheckIcon} title="Request and management access" description="New household users may submit up to ten active requests. Approval, transfer management and indexer administration are separate permissions.">
  <ResourceGate status={resource.status} error={resource.error} what="acquisition permissions" onretry={resource.load}>
    <Field.Group><Field.Field><Field.Label for="acquisition-user">User</Field.Label><Select.Root type="single" items={options} bind:value={selected}><Select.Trigger id="acquisition-user" class="w-full"><Select.Value placeholder="Choose a user" /></Select.Trigger><Select.Content><Select.Group>{#each options as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content></Select.Root></Field.Field>{#if user}{#key user.id}<PermissionFields {user} />{/key}{/if}</Field.Group>
  </ResourceGate>
</SettingsSection>
