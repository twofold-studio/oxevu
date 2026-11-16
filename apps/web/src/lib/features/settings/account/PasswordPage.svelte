<script lang="ts">
  import KeyRoundIcon from "@lucide/svelte/icons/key-round";
  import SettingsFrame from "../layout/SettingsFrame.svelte";
  import SettingsRow from "../layout/SettingsRow.svelte";
  import { SETTINGS_GROUP } from "../layout/group";
  import PasswordForm from "./PasswordForm.svelte";
</script>

<SettingsFrame page="account/password" description="Change the password you sign in with.">
  <div class={SETTINGS_GROUP}>
    <SettingsRow icon={KeyRoundIcon} title="Password" description="Changing it keeps you signed in on this device.">
      <PasswordForm />
    </SettingsRow>
  </div>
</SettingsFrame>
