<script lang="ts">
  import FolderInputIcon from "@lucide/svelte/icons/folder-input";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import { acquisitionDocument } from "$lib/features/settings/documents";
  import { createSettingsForm } from "$lib/features/settings/settings-form.svelte";
  import SettingsFrame from "$lib/features/settings/layout/SettingsFrame.svelte";
  import SettingsFormBody from "$lib/features/settings/layout/SettingsFormBody.svelte";
  import SettingsRow from "$lib/features/settings/layout/SettingsRow.svelte";
  import SwitchField from "$lib/features/settings/playback/SwitchField.svelte";
  import { validateAcquisition } from "./acquisition-form";

  const form = createSettingsForm(acquisitionDocument, ["import_enabled", "copy_fallback"], validateAcquisition);
</script>

<SettingsFrame
  page="downloads/imports"
  description="Whether completed downloads may be written into libraries. Library folders stay read-only unless this is on."
>
  <SettingsFormBody {form} what="download settings">
    {#if form.draft}
      <SettingsRow
        icon={FolderInputIcon}
        title="Library writes"
        description="Imports are reviewed per download or follow a title's destination rule."
      >
        <SwitchField
          id="acq-imports"
          label="Allow library imports"
          description="Permit reviewed imports and configured per-title destination rules to write completed media into libraries."
          bind:checked={form.draft.import_enabled}
        />
      </SettingsRow>
      <SettingsRow
        icon={CopyIcon}
        title="Across filesystems"
        description="Imports hardlink by default so the seeding copy and the library copy share space."
      >
        <SwitchField
          id="acq-copy"
          label="Cross-filesystem copy fallback"
          description="If a hardlink is impossible across filesystems, copy and verify the file. The seeding source remains."
          bind:checked={form.draft.copy_fallback}
        />
      </SettingsRow>
    {/if}
  </SettingsFormBody>
</SettingsFrame>
