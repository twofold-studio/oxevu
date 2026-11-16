<script lang="ts">
  import GaugeIcon from "@lucide/svelte/icons/gauge";
  import ListOrderedIcon from "@lucide/svelte/icons/list-ordered";
  import * as Field from "$lib/components/ui/field";
  import { playbackDocument } from "../documents";
  import { createSettingsForm } from "../settings-form.svelte";
  import SettingsFrame from "../layout/SettingsFrame.svelte";
  import SettingsFormBody from "../layout/SettingsFormBody.svelte";
  import SettingsRow from "../layout/SettingsRow.svelte";
  import NumberField from "./NumberField.svelte";
  import { validateSettings } from "./settings-form";

  const form = createSettingsForm(
    playbackDocument,
    ["max_concurrent_transcodes", "max_video_bitrate_mbps", "max_height", "cpu_crf"],
    (draft, envelope) => validateSettings(draft, envelope.capabilities),
  );
</script>

<SettingsFrame
  page="playback/quality"
  description="How much conversion work runs at once and the ceiling on what it produces. Active streams keep the settings they started with."
>
  <SettingsFormBody {form} what="playback settings" saved="Changes saved. They apply to new streams.">
    {#if form.draft}
      <SettingsRow icon={ListOrderedIcon} title="Concurrency" description="Further requests wait or fail until one ends.">
        <NumberField
          id="max-transcodes"
          label="Simultaneous conversions"
          bind:value={form.draft.max_concurrent_transcodes}
          error={form.errors.max_concurrent_transcodes}
          min={1}
          max={16}
        />
      </SettingsRow>
      <SettingsRow icon={GaugeIcon} title="Output quality" description="Upper bounds for converted video.">
        <Field.Group>
          <NumberField
            id="max-bitrate"
            label="Maximum video bitrate"
            unit="Mbps"
            step={0.25}
            bind:value={form.draft.max_video_bitrate_mbps}
            error={form.errors.max_video_bitrate_mbps}
            min={0.25}
            max={200}
          />
          <NumberField
            id="max-height"
            label="Maximum height"
            description="Taller sources are scaled down, e.g. 1080 or 2160."
            unit="px"
            bind:value={form.draft.max_height}
            error={form.errors.max_height}
            min={144}
            max={4320}
          />
          <NumberField
            id="cpu-crf"
            label="CPU quality (CRF)"
            description="Lower is better quality and larger. 23 is a good default."
            bind:value={form.draft.cpu_crf}
            error={form.errors.cpu_crf}
            min={0}
            max={51}
          />
        </Field.Group>
      </SettingsRow>
    {/if}
  </SettingsFormBody>
</SettingsFrame>
