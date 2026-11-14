<script lang="ts">
  import * as Field from "$lib/components/ui/field";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import type { Capabilities, HardwareAcceleration, Settings } from "$lib/api/admin-types";
  import type { SettingsErrors } from "./settings-form";
  import NumberField from "./NumberField.svelte";
  import SwitchField from "./SwitchField.svelte";

  let {
    draft = $bindable(),
    errors,
    capabilities,
  }: { draft: Settings; errors: SettingsErrors; capabilities: Capabilities } = $props();

  const nvenc = $derived(draft.hardware_acceleration === "nvenc");
  const off = $derived(!draft.transcoding_enabled);
</script>

<Field.Group>
  <SwitchField
    id="transcoding-enabled"
    label="Allow conversion"
    description="When off, titles the browser can't play directly won't play. Repackaging also counts as conversion."
    bind:checked={draft.transcoding_enabled}
  />

  <Field.Set data-disabled={off ? true : undefined}>
    <Field.Legend variant="label">Hardware acceleration</Field.Legend>
    <ToggleGroup.Root
      type="single"
      variant="outline"
      disabled={off}
      bind:value={() => draft.hardware_acceleration,
      (v) => v && (draft.hardware_acceleration = v as HardwareAcceleration)}
      aria-label="Hardware acceleration"
    >
      <ToggleGroup.Item value="none">None (CPU)</ToggleGroup.Item>
      <ToggleGroup.Item value="nvenc">NVIDIA NVENC</ToggleGroup.Item>
    </ToggleGroup.Root>
    {#if nvenc && !capabilities.nvenc_available}
      <Field.Description class="text-destructive">
        The server didn't detect a working NVENC encoder. Sessions will use the CPU if fallback is allowed, or fail.
      </Field.Description>
    {:else}
      <Field.Description>
        Other GPU vendors aren't supported yet. HDR conversion is unavailable; the server rejects HDR conversion
        requests because tone mapping is not implemented.
      </Field.Description>
    {/if}
  </Field.Set>

  {#if nvenc}
    <SwitchField
      id="hardware-encoding"
      label="Hardware encoding"
      description="Encode H.264 on the GPU."
      disabled={off}
      bind:checked={draft.hardware_encoding}
    />
    <SwitchField
      id="hardware-decoding"
      label="Hardware decoding"
      description={capabilities.hardware_decode_available
        ? "Decode the source on the GPU as well."
        : "Not available yet: this server decodes on the CPU and rejects this setting."}
      disabled={off || !capabilities.hardware_decode_available}
      error={errors.hardware_decoding}
      bind:checked={draft.hardware_decoding}
    />
    <NumberField
      id="gpu-index"
      label="GPU"
      description="Device index as numbered by the NVIDIA driver; 0 is the first GPU."
      disabled={off}
      bind:value={draft.gpu_index}
      error={errors.gpu_index}
      min={0}
      max={31}
    />
  {/if}

  <SwitchField
    id="cpu-fallback"
    label="Fall back to CPU"
    description="If the GPU fails to start a stream, retry once on the CPU. The player shows which one was used."
    disabled={off}
    bind:checked={draft.allow_cpu_fallback}
  />
</Field.Group>
