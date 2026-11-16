<script lang="ts">
  import GaugeIcon from "@lucide/svelte/icons/gauge";
  import { acquisitionDocument } from "$lib/features/settings/documents";
  import { createSettingsForm } from "$lib/features/settings/settings-form.svelte";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import SaveStatus from "$lib/features/settings/layout/SaveStatus.svelte";
  import * as Field from "$lib/components/ui/field";
  import NumberField from "$lib/features/settings/playback/NumberField.svelte";
  import { validateAcquisition } from "./settings/acquisition-form";

  const form = createSettingsForm(
    acquisitionDocument,
    ["download_limit_kib", "upload_limit_kib"],
    validateAcquisition,
  );
</script>

<section class="mb-6 flex flex-col gap-4 rounded-xl border bg-card p-4" aria-label="Transfer speed limits">
  <div>
    <h2 class="flex items-center gap-2 text-sm font-medium"><GaugeIcon class="size-4" />Speed limits</h2>
    <p class="mt-1 text-xs text-muted-foreground">Shared by all transfers. Zero is unlimited; other limits start at 16 KiB/s. Changes apply immediately without restarting downloads.</p>
  </div>
  <ResourceGate status={form.status} error={form.loadError} what="speed limits" onretry={form.load}>
    {#if form.draft}
      <form onsubmit={(event) => event.preventDefault()} novalidate>
        <Field.Group class="grid max-w-xl gap-4 sm:grid-cols-2">
          <NumberField id="activity-download-limit" label="Download" unit="KiB/s" min={0} max={1000000} bind:value={form.draft.download_limit_kib} error={form.errors.download_limit_kib} />
          <NumberField id="activity-upload-limit" label="Upload" unit="KiB/s" min={0} max={1000000} bind:value={form.draft.upload_limit_kib} error={form.errors.upload_limit_kib} />
        </Field.Group>
      </form>
      <SaveStatus phase={form.phase} failure={form.failure} invalid={form.invalid} />
    {/if}
  </ResourceGate>
</section>
