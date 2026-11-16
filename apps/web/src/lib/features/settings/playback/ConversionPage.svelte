<script lang="ts">
  import ActivityIcon from "@lucide/svelte/icons/activity";
  import CpuIcon from "@lucide/svelte/icons/cpu";
  import { playbackDocument } from "../documents";
  import { createSettingsForm } from "../settings-form.svelte";
  import SettingsFrame from "../layout/SettingsFrame.svelte";
  import SettingsFormBody from "../layout/SettingsFormBody.svelte";
  import SettingsRow from "../layout/SettingsRow.svelte";
  import CapabilityPanel from "./CapabilityPanel.svelte";
  import EncoderFields from "./EncoderFields.svelte";
  import { validateSettings } from "./settings-form";

  const form = createSettingsForm(
    playbackDocument,
    [
      "transcoding_enabled",
      "hardware_acceleration",
      "hardware_encoding",
      "hardware_decoding",
      "gpu_index",
      "allow_cpu_fallback",
    ],
    (draft, envelope) => validateSettings(draft, envelope.capabilities),
  );
</script>

<SettingsFrame
  page="playback/conversion"
  description="Used when a browser can't play the original file, or a viewer chooses Convert. Output is H.264 video with AAC audio."
>
  <SettingsFormBody {form} what="playback settings" saved="Changes saved. They apply to new streams.">
    {#if form.draft && form.envelope}
      <SettingsRow
        icon={ActivityIcon}
        title="Server capabilities"
        description="Detected when the server starts. Conversion needs FFmpeg; NVIDIA acceleration needs a working NVENC encoder."
      >
        <CapabilityPanel capabilities={form.envelope.capabilities} />
      </SettingsRow>
      <SettingsRow
        icon={CpuIcon}
        title="Encoder"
        description="Whether conversion is allowed and which hardware runs it."
      >
        <EncoderFields bind:draft={form.draft} capabilities={form.envelope.capabilities} errors={form.errors} />
      </SettingsRow>
    {/if}
  </SettingsFormBody>
</SettingsFrame>
