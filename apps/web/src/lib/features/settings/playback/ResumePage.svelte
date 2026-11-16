<script lang="ts">
  import HistoryIcon from "@lucide/svelte/icons/history";
  import { playbackDocument } from "../documents";
  import { createSettingsForm } from "../settings-form.svelte";
  import SettingsFrame from "../layout/SettingsFrame.svelte";
  import SettingsFormBody from "../layout/SettingsFormBody.svelte";
  import SettingsRow from "../layout/SettingsRow.svelte";
  import ResumeFields from "./ResumeFields.svelte";
  import { validateSettings } from "./settings-form";

  const form = createSettingsForm(
    playbackDocument,
    ["resume_min_percent", "resume_max_percent", "resume_min_duration_seconds"],
    (draft, envelope) => validateSettings(draft, envelope.capabilities),
  );
</script>

<SettingsFrame
  page="playback/resume"
  description="When a partly watched title offers to continue, and when it counts as watched. Applies to every user."
>
  <SettingsFormBody {form} what="playback settings">
    {#if form.draft}
      <SettingsRow
        icon={HistoryIcon}
        title="Resume points"
        description="Progress outside this window starts over or marks the title as watched."
      >
        <ResumeFields bind:draft={form.draft} errors={form.errors} />
      </SettingsRow>
    {/if}
  </SettingsFormBody>
</SettingsFrame>
