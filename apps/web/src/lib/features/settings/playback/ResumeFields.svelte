<script lang="ts">
  import * as Field from "$lib/components/ui/field";
  import type { Settings } from "$lib/api/admin-types";
  import type { SettingsErrors } from "./settings-form";
  import NumberField from "./NumberField.svelte";

  let { draft = $bindable(), errors }: { draft: Settings; errors: SettingsErrors } = $props();
</script>

<Field.Group>
  <NumberField
    id="resume-min"
    label="Minimum resume percentage"
    description="Stopping earlier than this starts over next time."
    unit="%"
    bind:value={draft.resume_min_percent}
    error={errors.resume_min_percent}
    min={0}
    max={99}
  />
  <NumberField
    id="resume-max"
    label="Maximum resume percentage"
    description="Stopping later than this marks the title as watched."
    unit="%"
    bind:value={draft.resume_max_percent}
    error={errors.resume_max_percent}
    min={1}
    max={100}
  />
  <NumberField
    id="resume-duration"
    label="Shortest resumable length"
    description="Shorter titles always start over. 300 s is five minutes."
    unit="s"
    bind:value={draft.resume_min_duration_seconds}
    error={errors.resume_min_duration_seconds}
    min={0}
    max={86400}
  />
</Field.Group>
