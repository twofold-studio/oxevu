<script lang="ts">
  import * as Sheet from "$lib/components/ui/sheet";
  import * as Field from "$lib/components/ui/field";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { usersApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import type { UserSettings } from "$lib/api/admin-types";
  import { MIN_PASSWORD_BYTES, type Library, type SyncplayAccess } from "$lib/api/types";
  import SwitchField from "../playback/SwitchField.svelte";
  import LibraryAccess from "./LibraryAccess.svelte";
  import { draftFromUser, hasUserErrors, newUserDraft, toUserInput, validateUser, type UserDraft } from "./user-form";

  // Create (no `user`) or edit an account. The parent remounts this per
  // opening. The server protects the last enabled administrator and signs
  // out sessions whose access changed.
  let {
    open = $bindable(false),
    user = null,
    self,
    libraries,
    onsaved,
  }: {
    open: boolean;
    user?: UserSettings | null;
    /** Editing your own account. */
    self: boolean;
    libraries: Library[];
    onsaved: (user: UserSettings, created: boolean) => void;
  } = $props();

  // svelte-ignore state_referenced_locally
  let draft = $state<UserDraft>(user ? draftFromUser(user) : newUserDraft());
  let submitted = $state(false);
  let pending = $state(false);
  let failure = $state<string | null>(null);

  const creating = $derived(!user);
  const errors = $derived(validateUser(draft, creating));
  const show = (message: string | null) => (submitted ? message : null);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    submitted = true;
    failure = null;
    if (hasUserErrors(errors) || pending) return;
    pending = true;
    try {
      const body = toUserInput(draft, libraries.map((library) => library.id));
      const saved = user ? (await usersApi.update(user.id, body)).user : (await usersApi.create(body)).user;
      onsaved(saved, !user);
      open = false;
    } catch (error) {
      failure = errorMessage(error);
    } finally {
      pending = false;
    }
  }
</script>

<Sheet.Root bind:open>
  <Sheet.Content class="w-full overflow-y-auto sm:max-w-md">
    <Sheet.Header>
      <Sheet.Title>{user ? `Edit ${user.username}` : "Add a user"}</Sheet.Title>
      <Sheet.Description>
        {user ? "Changes to role or access sign this account out of its other sessions." : "New accounts can sign in right away. There is no public sign-up."}
      </Sheet.Description>
    </Sheet.Header>
    <form class="px-4 pb-4" onsubmit={submit} novalidate>
      <Field.Group>
        <Field.Field data-invalid={show(errors.username) ? true : undefined}>
          <Field.Label for="user-name">Username</Field.Label>
          <Input id="user-name" bind:value={draft.username} autocomplete="off" autocapitalize="none" spellcheck={false} aria-invalid={show(errors.username) ? true : undefined} />
          {#if show(errors.username)}<Field.Error>{errors.username}</Field.Error>{/if}
        </Field.Field>

        <Field.Field data-invalid={show(errors.password) ? true : undefined}>
          <Field.Label for="user-password">{user ? "New password" : "Password"}</Field.Label>
          <Input id="user-password" type="password" bind:value={draft.password} autocomplete="new-password" aria-invalid={show(errors.password) ? true : undefined} />
          {#if show(errors.password)}
            <Field.Error>{errors.password}</Field.Error>
          {:else}
            <Field.Description>
              {user ? "Leave blank to keep the current password." : `At least ${MIN_PASSWORD_BYTES} characters.`}
            </Field.Description>
          {/if}
        </Field.Field>

        <Field.Separator />

        <SwitchField
          id="user-admin"
          label="Administrator"
          description={self ? "You can't remove your own administrator role here if you're the last one." : "Manages libraries, playback settings and users."}
          bind:checked={draft.is_admin}
        />
        <SwitchField
          id="user-disabled"
          label="Disabled"
          description="Signs the account out everywhere and blocks sign-in, playback and watch parties."
          bind:checked={draft.disabled}
        />
        <SwitchField id="user-play" label="Can play" description="Watch titles in allowed libraries." bind:checked={draft.can_play} />
        <SwitchField
          id="user-transcode"
          label="Can use conversion"
          description="Without it, only files the browser plays directly will work."
          bind:checked={draft.can_transcode}
        />

        <Field.Set>
          <Field.Legend variant="label">Watch parties</Field.Legend>
          <ToggleGroup.Root
            type="single"
            variant="outline"
            bind:value={() => draft.syncplay_access, (v) => v && (draft.syncplay_access = v as SyncplayAccess)}
            aria-label="Watch party access"
          >
            <ToggleGroup.Item value="create_join">Create and join</ToggleGroup.Item>
            <ToggleGroup.Item value="join">Join</ToggleGroup.Item>
            <ToggleGroup.Item value="disabled">Off</ToggleGroup.Item>
          </ToggleGroup.Root>
          <Field.Description>Parties never grant access to libraries the account can't open.</Field.Description>
        </Field.Set>

        <Field.Separator />

        <LibraryAccess {libraries} isAdmin={draft.is_admin} bind:allLibraries={draft.all_libraries} bind:libraryIds={draft.library_ids} />

        {#if failure}<p class="text-sm text-destructive" role="alert">{failure}</p>{/if}

        <div class="flex justify-end gap-2">
          <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
          <Button type="submit" disabled={pending}>
            {#if pending}<Spinner data-icon="inline-start" />{/if}
            {user ? "Save changes" : "Add user"}
          </Button>
        </div>
      </Field.Group>
    </form>
  </Sheet.Content>
</Sheet.Root>
