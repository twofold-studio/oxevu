<script lang="ts">
  import ArrowUpFromLineIcon from "@lucide/svelte/icons/arrow-up-from-line";
  import * as Field from "$lib/components/ui/field";
  import { acquisitionDocument } from "$lib/features/settings/documents";
  import { createSettingsForm } from "$lib/features/settings/settings-form.svelte";
  import SettingsFrame from "$lib/features/settings/layout/SettingsFrame.svelte";
  import SettingsFormBody from "$lib/features/settings/layout/SettingsFormBody.svelte";
  import SettingsRow from "$lib/features/settings/layout/SettingsRow.svelte";
  import NumberField from "$lib/features/settings/playback/NumberField.svelte";
  import { validateAcquisition } from "./acquisition-form";

  const form = createSettingsForm(acquisitionDocument, ["seed_ratio", "seed_hours"], validateAcquisition);
</script>

<SettingsFrame
  page="downloads/seeding"
  description="When completed transfers stop uploading. Reaching the targets pauses seeding without deleting files."
>
  <SettingsFormBody {form} what="download settings">
    {#if form.draft}
      <SettingsRow
        icon={ArrowUpFromLineIcon}
        title="Seeding targets"
        description="Zero disables a target. When both are enabled, both must be met."
      >
        <Field.Group>
          <NumberField
            id="acq-seed_ratio"
            label="Seed ratio target"
            description="Uploaded bytes divided by downloaded bytes."
            step={0.1}
            min={0}
            max={100}
            bind:value={form.draft.seed_ratio}
            error={form.errors.seed_ratio}
          />
          <NumberField
            id="acq-seed_hours"
            label="Seed time target"
            unit="hours"
            min={0}
            max={87600}
            bind:value={form.draft.seed_hours}
            error={form.errors.seed_hours}
          />
        </Field.Group>
      </SettingsRow>
    {/if}
  </SettingsFormBody>
</SettingsFrame>
