<script lang="ts">
  import MonitorPlayIcon from "@lucide/svelte/icons/monitor-play";
  import SettingsFrame from "../layout/SettingsFrame.svelte";
  import SettingsRow from "../layout/SettingsRow.svelte";
  import { SETTINGS_GROUP } from "../layout/group";
  import SeekPreference from "./SeekPreference.svelte";
</script>

<SettingsFrame page="account/player" description="Player preferences stored in this browser only.">
  <div class={SETTINGS_GROUP}>
    <SettingsRow icon={MonitorPlayIcon} title="Seeking" description="How far one skip moves.">
      <SeekPreference />
    </SettingsRow>
  </div>
</SettingsFrame>
