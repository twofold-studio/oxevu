<script lang="ts">
  import PowerIcon from "@lucide/svelte/icons/power";
  import FolderIcon from "@lucide/svelte/icons/folder";
  import HardDriveIcon from "@lucide/svelte/icons/hard-drive";
  import * as Field from "$lib/components/ui/field";
  import { acquisitionDocument } from "$lib/features/settings/documents";
  import { createSettingsForm } from "$lib/features/settings/settings-form.svelte";
  import SettingsFrame from "$lib/features/settings/layout/SettingsFrame.svelte";
  import SettingsFormBody from "$lib/features/settings/layout/SettingsFormBody.svelte";
  import SettingsRow from "$lib/features/settings/layout/SettingsRow.svelte";
  import NumberField from "$lib/features/settings/playback/NumberField.svelte";
  import SwitchField from "$lib/features/settings/playback/SwitchField.svelte";
  import PathInput from "$lib/features/settings/libraries/PathInput.svelte";
  import { validateAcquisition } from "./acquisition-form";

  const form = createSettingsForm(
    acquisitionDocument,
    ["enabled", "download_dir", "storage_limit_gb", "min_free_gb"],
    validateAcquisition,
  );
</script>

<SettingsFrame
  page="downloads/storage"
  description="Whether the embedded torrent engine runs, and where and how much it may store. Changing the engine state or folder restarts it and rechecks managed files."
>
  <SettingsFormBody {form} what="download settings">
    {#if form.draft}
      <SettingsRow
        icon={PowerIcon}
        title="Engine"
        description={form.envelope?.engine ? `Embedded ${form.envelope.engine}.` : "Embedded BitTorrent engine."}
      >
        <SwitchField
          id="acq-enabled"
          label="Enable downloads"
          description="Run the embedded torrent engine. Direct traffic is allowed; no VPN is required."
          bind:checked={form.draft.enabled}
        />
      </SettingsRow>
      <SettingsRow
        icon={FolderIcon}
        title="Download folder"
        description="Where transfers are written before they are imported into a library."
      >
        <Field.Field data-invalid={form.errors.download_dir ? true : undefined}>
          <Field.Label>Download folder</Field.Label>
          <PathInput bind:value={form.draft.download_dir} label="Download folder" invalid={Boolean(form.errors.download_dir)} />
          {#if form.errors.download_dir}
            <Field.Error>{form.errors.download_dir}</Field.Error>
          {:else}
            <Field.Description>Keep downloads separate from library roots, source code and conversion caches. It can't change while transfers exist.</Field.Description>
          {/if}
        </Field.Field>
      </SettingsRow>
      <SettingsRow
        icon={HardDriveIcon}
        title="Storage limits"
        description="Transfers pause when either limit is reached."
      >
        <Field.Group>
          <NumberField
            id="acq-storage_limit_gb"
            label="Download storage ceiling"
            description="Transfers pause when sampled storage usage reaches the limit."
            unit="GiB"
            min={1}
            max={100000}
            bind:value={form.draft.storage_limit_gb}
            error={form.errors.storage_limit_gb}
          />
          <NumberField
            id="acq-min_free_gb"
            label="Keep free"
            description="Reserve room on the disk for playback and other services."
            unit="GiB"
            min={0}
            max={100000}
            bind:value={form.draft.min_free_gb}
            error={form.errors.min_free_gb}
          />
        </Field.Group>
      </SettingsRow>
    {/if}
  </SettingsFormBody>
</SettingsFrame>
