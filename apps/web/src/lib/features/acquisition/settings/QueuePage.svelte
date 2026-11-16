<script lang="ts">
  import ListOrderedIcon from "@lucide/svelte/icons/list-ordered";
  import GaugeIcon from "@lucide/svelte/icons/gauge";
  import SearchIcon from "@lucide/svelte/icons/search";
  import * as Field from "$lib/components/ui/field";
  import { acquisitionDocument } from "$lib/features/settings/documents";
  import { createSettingsForm } from "$lib/features/settings/settings-form.svelte";
  import SettingsFrame from "$lib/features/settings/layout/SettingsFrame.svelte";
  import SettingsFormBody from "$lib/features/settings/layout/SettingsFormBody.svelte";
  import SettingsRow from "$lib/features/settings/layout/SettingsRow.svelte";
  import NumberField from "$lib/features/settings/playback/NumberField.svelte";
  import { validateAcquisition } from "./acquisition-form";

  const form = createSettingsForm(
    acquisitionDocument,
    ["max_active_downloads", "download_limit_kib", "upload_limit_kib", "search_interval_minutes"],
    validateAcquisition,
  );
  const speed = "Zero is unlimited. Nonzero limits must be at least 16 KiB/s.";
</script>

<SettingsFrame
  page="downloads/queue"
  description="How many transfers run at once, how fast they may go, and how often missing titles are searched."
>
  <SettingsFormBody {form} what="download settings">
    {#if form.draft}
      <SettingsRow icon={ListOrderedIcon} title="Queue" description="Further transfers wait their turn.">
        <NumberField
          id="acq-max_active_downloads"
          label="Active downloads"
          description="Completed torrents can continue seeding."
          min={1}
          max={16}
          bind:value={form.draft.max_active_downloads}
          error={form.errors.max_active_downloads}
        />
      </SettingsRow>
      <SettingsRow icon={GaugeIcon} title="Speed limits" description="Shared by all transfers. Applied without a restart.">
        <Field.Group>
          <NumberField
            id="acq-download_limit_kib"
            label="Download speed limit"
            description={speed}
            unit="KiB/s"
            min={0}
            max={1000000}
            bind:value={form.draft.download_limit_kib}
            error={form.errors.download_limit_kib}
          />
          <NumberField
            id="acq-upload_limit_kib"
            label="Upload speed limit"
            description={speed}
            unit="KiB/s"
            min={0}
            max={1000000}
            bind:value={form.draft.upload_limit_kib}
            error={form.errors.upload_limit_kib}
          />
        </Field.Group>
      </SettingsRow>
      <SettingsRow
        icon={SearchIcon}
        title="Missing-title search"
        description="Applies only to titles with automatic acquisition enabled under Wanted."
      >
        <NumberField
          id="acq-search_interval_minutes"
          label="Search interval"
          unit="min"
          min={5}
          max={1440}
          bind:value={form.draft.search_interval_minutes}
          error={form.errors.search_interval_minutes}
        />
      </SettingsRow>
    {/if}
  </SettingsFormBody>
</SettingsFrame>
