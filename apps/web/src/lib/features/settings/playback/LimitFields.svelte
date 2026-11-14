<script lang="ts">
  import * as Field from "$lib/components/ui/field";
  import type { Settings } from "$lib/api/admin-types";
  import type { SettingsErrors } from "./settings-form";
  import NumberField from "./NumberField.svelte";

  let { draft = $bindable(), errors }: { draft: Settings; errors: SettingsErrors } = $props();
</script>

<Field.Group class="grid gap-6 sm:grid-cols-2">
  <NumberField
    id="max-transcodes"
    label="Simultaneous conversions"
    description="Further requests wait or fail until one ends."
    bind:value={draft.max_concurrent_transcodes}
    error={errors.max_concurrent_transcodes}
    min={1}
    max={16}
  />
  <NumberField
    id="max-bitrate"
    label="Maximum video bitrate"
    unit="Mbps"
    step={0.25}
    bind:value={draft.max_video_bitrate_mbps}
    error={errors.max_video_bitrate_mbps}
    min={0.25}
    max={200}
  />
  <NumberField
    id="max-height"
    label="Maximum height"
    description="Taller sources are scaled down, e.g. 1080 or 2160."
    unit="px"
    bind:value={draft.max_height}
    error={errors.max_height}
    min={144}
    max={4320}
  />
  <NumberField
    id="cpu-crf"
    label="CPU quality (CRF)"
    description="Lower is better quality and larger. 23 is a good default."
    bind:value={draft.cpu_crf}
    error={errors.cpu_crf}
    min={0}
    max={51}
  />
  <NumberField
    id="scratch-limit"
    label="Scratch space limit"
    description="Converted segments are deleted when sessions end."
    unit="GB"
    step={0.5}
    bind:value={draft.scratch_limit_gb}
    error={errors.scratch_limit_gb}
    min={0.1}
    max={1024}
  />
  <NumberField
    id="session-idle"
    label="Stop idle streams after"
    description="A stream whose player stopped reporting is ended and cleaned up."
    unit="s"
    bind:value={draft.session_idle_seconds}
    error={errors.session_idle_seconds}
    min={30}
    max={3600}
  />
</Field.Group>
