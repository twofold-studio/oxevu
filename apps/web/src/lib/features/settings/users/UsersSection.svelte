<script lang="ts">
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import UsersIcon from "@lucide/svelte/icons/users";
  import UserPlusIcon from "@lucide/svelte/icons/user-plus";
  import { Button } from "$lib/components/ui/button";
  import { librariesApi, usersApi } from "$lib/api/endpoints";
  import type { UserSettings } from "$lib/api/admin-types";
  import { session } from "$lib/state/session.svelte";
  import ResourceGate from "../ResourceGate.svelte";
  import SectionHeading from "../SectionHeading.svelte";
  import { createResource } from "../resource.svelte";
  import UserRow from "./UserRow.svelte";
  import UserDialog from "./UserDialog.svelte";

  // Users and libraries load together: grants are edited against the list.
  const data = createResource(async () => {
    const [{ users }, { libraries }] = await Promise.all([usersApi.list(), librariesApi.list()]);
    return { users, libraries };
  });

  let editorOpen = $state(false);
  let editing = $state<UserSettings | null>(null);
  let opening = $state(0);

  onMount(() => {
    void data.load();
  });

  function openEditor(user: UserSettings | null) {
    editing = user;
    opening++;
    editorOpen = true;
  }

  function saved(user: UserSettings, created: boolean) {
    if (!data.value) return;
    const users = created ? [...data.value.users, user] : data.value.users.map((u) => (u.id === user.id ? user : u));
    data.value = { ...data.value, users };
    toast.success(created ? `Added ${user.username}.` : `Saved ${user.username}.`);
    // Your own role or access may have changed; the server may also have
    // ended this session, which the next request reports.
    if (user.id === session.user?.id) void session.load();
  }
</script>

<section class="flex flex-col gap-4" aria-labelledby="users-heading">
  <SectionHeading
    id="users-heading"
    icon={UsersIcon}
    title="Users"
    description="Accounts that can sign in, what they may watch, and whether they can use conversion and watch parties."
  >
    {#snippet action()}
      {#if data.status === "ready"}
        <Button size="sm" onclick={() => openEditor(null)}>
          <UserPlusIcon data-icon="inline-start" />
          Add user
        </Button>
      {/if}
    {/snippet}
  </SectionHeading>

  <ResourceGate status={data.status} error={data.error} what="users" onretry={data.load}>
    {#if data.value}
      <ul class="divide-y rounded-xl border bg-card">
        {#each data.value.users as user (user.id)}
          <UserRow
            {user}
            self={user.id === session.user?.id}
            libraries={data.value.libraries}
            onedit={() => openEditor(user)}
          />
        {/each}
      </ul>
    {/if}
  </ResourceGate>
</section>

{#key opening}
  {#if data.value}
    <UserDialog
      bind:open={editorOpen}
      user={editing}
      self={editing?.id === session.user?.id}
      libraries={data.value.libraries}
      onsaved={saved}
    />
  {/if}
{/key}
