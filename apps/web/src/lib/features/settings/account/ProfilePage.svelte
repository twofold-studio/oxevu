<script lang="ts">
  import UserIcon from "@lucide/svelte/icons/user-round";
  import { session } from "$lib/state/session.svelte";
  import SettingsFrame from "../layout/SettingsFrame.svelte";
  import SettingsRow from "../layout/SettingsRow.svelte";
  import { SETTINGS_GROUP } from "../layout/group";

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

<SettingsFrame page="account/profile" description="Who you are signed in as and what this account may do.">
  <div class={SETTINGS_GROUP}>
    <SettingsRow
      icon={UserIcon}
      title="Your access"
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
    </SettingsRow>
  </div>
</SettingsFrame>
