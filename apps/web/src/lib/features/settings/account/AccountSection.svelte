<script lang="ts">
  import UserIcon from "@lucide/svelte/icons/user-round";
  import KeyRoundIcon from "@lucide/svelte/icons/key-round";
  import { session } from "$lib/state/session.svelte";
  import SettingsSection from "../SettingsSection.svelte";
  import PasswordForm from "./PasswordForm.svelte";

  const user = $derived(session.user);
  const rows = $derived(
    user
      ? [
          ["Username", user.username],
          ["Role", user.is_admin ? "Administrator" : "Viewer"],
          ["Playback", !user.can_play ? "Not allowed" : user.can_transcode ? "Original files and conversion" : "Original files only"],
          [
            "Watch parties",
            user.syncplay_access === "create_join" ? "Create and join" : user.syncplay_access === "join" ? "Join only" : "Off",
          ],
        ]
      : [],
  );
</script>

<div class="flex flex-col gap-4">
  <SettingsSection
    icon={UserIcon}
    title="Your account"
    description="Set by an administrator. Ask one if you need different access."
  >
    <dl class="divide-y rounded-lg border text-sm">
      {#each rows as [label, value] (label)}
        <div class="flex items-center justify-between gap-4 px-4 py-2.5">
          <dt class="text-muted-foreground">{label}</dt>
          <dd class="truncate text-right">{value}</dd>
        </div>
      {/each}
    </dl>
  </SettingsSection>

  <SettingsSection icon={KeyRoundIcon} title="Password" description="Changing it keeps you signed in on this device.">
    <PasswordForm />
  </SettingsSection>
</div>
