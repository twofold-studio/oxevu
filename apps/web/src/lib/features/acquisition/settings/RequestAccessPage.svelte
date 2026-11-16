<script lang="ts">
  import { onMount } from "svelte";
  import ShieldCheckIcon from "@lucide/svelte/icons/shield-check";
  import * as Field from "$lib/components/ui/field";
  import * as Select from "$lib/components/ui/select";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import SettingsFrame from "$lib/features/settings/layout/SettingsFrame.svelte";
  import SettingsRow from "$lib/features/settings/layout/SettingsRow.svelte";
  import { SETTINGS_GROUP } from "$lib/features/settings/layout/group";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import { usersApi } from "$lib/api/endpoints";
  import { href } from "$lib/state/router.svelte";
  import PermissionFields from "./PermissionFields.svelte";

  const resource = createResource(async () => (await usersApi.list()).users);
  let selected = $state("");
  const options = $derived((resource.value ?? []).map((u) => ({ value: u.id, label: u.username })));
  const user = $derived(resource.value?.find((u) => u.id === selected));

  onMount(() => {
    void resource.load();
  });
</script>

<SettingsFrame
  page="people/requests"
  description="Who may request media, how many requests they may have open, and who may review requests, manage transfers or configure indexers."
>
  <ResourceGate status={resource.status} error={resource.error} what="request access" onretry={resource.load}>
    <div class={SETTINGS_GROUP}>
      <SettingsRow
        icon={ShieldCheckIcon}
        title="Per-user access"
        description="New household users may submit up to ten active requests. Library and playback access are set under Users."
      >
        <Field.Group>
          <Field.Field>
            <Field.Label for="acquisition-user">User</Field.Label>
            <Select.Root type="single" items={options} bind:value={selected}>
              <Select.Trigger id="acquisition-user" class="w-full"><Select.Value placeholder="Choose a user" /></Select.Trigger>
              <Select.Content>
                <Select.Group>
                  {#each options as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}
                </Select.Group>
              </Select.Content>
            </Select.Root>
            {#if !user}
              <Field.Description>
                Choose an account to edit its permissions. <a class="underline underline-offset-4" href={href.settings("people/users")}>Manage users</a>
              </Field.Description>
            {/if}
          </Field.Field>
          {#if user}
            {#key user.id}<PermissionFields {user} />{/key}
          {/if}
        </Field.Group>
      </SettingsRow>
    </div>
  </ResourceGate>
</SettingsFrame>
