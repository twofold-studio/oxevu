<script lang="ts">
  import HardDriveIcon from "@lucide/svelte/icons/hard-drive";
  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import TimerIcon from "@lucide/svelte/icons/timer";
  import * as Field from "$lib/components/ui/field";
  import { playbackDocument } from "../documents";
  import { createSettingsForm } from "../settings-form.svelte";
  import SettingsFrame from "../layout/SettingsFrame.svelte";
  import SettingsFormBody from "../layout/SettingsFormBody.svelte";
  import SettingsRow from "../layout/SettingsRow.svelte";
  import NumberField from "./NumberField.svelte";
  import SwitchField from "./SwitchField.svelte";
  import { validateSettings } from "./settings-form";

  const form = createSettingsForm(
    playbackDocument,
    ["scratch_limit_gb", "session_idle_seconds", "cache_enabled", "cache_limit_gb"],
    (draft, envelope) => validateSettings(draft, envelope.capabilities),
  );
</script>

<SettingsFrame
  page="playback/scratch"
  description="Disk space conversion may use, how long abandoned streams live, and whether converted video is kept for reuse."
>
  <SettingsFormBody {form} what="playback settings" saved="Changes saved. They apply to new streams.">
    {#if form.draft && form.envelope}
      <SettingsRow
        icon={HardDriveIcon}
        title="Scratch space"
        description="Converted segments for active streams, deleted when sessions end."
      >
        <Field.Group>
          <NumberField
            id="scratch-limit"
            label="Scratch space limit"
            unit="GB"
            step={0.5}
            bind:value={form.draft.scratch_limit_gb}
            error={form.errors.scratch_limit_gb}
            min={0.1}
            max={1024}
          />
          <Field.Field>
            <Field.Label>Scratch folder · <code class="font-mono text-xs">OXEVU_TRANSCODE_DIR</code></Field.Label>
            <p class="font-mono text-xs break-all">{form.envelope.capabilities.scratch_path ?? "Not configured"}</p>
            <Field.Description>Set in the server's environment. It must be outside every library folder.</Field.Description>
          </Field.Field>
        </Field.Group>
      </SettingsRow>
      <SettingsRow
        icon={TimerIcon}
        title="Idle streams"
        description="A stream whose player stopped reporting is ended and cleaned up."
      >
        <NumberField
          id="session-idle"
          label="Stop idle streams after"
          unit="s"
          bind:value={form.draft.session_idle_seconds}
          error={form.errors.session_idle_seconds}
          min={30}
          max={3600}
        />
      </SettingsRow>
      <SettingsRow
        icon={ArchiveIcon}
        title="Conversion cache"
        description="Persistent cache outside media folders; separate from active scratch space."
      >
        <Field.Group>
          <SwitchField
            id="conversion-cache"
            label="Cache converted video"
            description="Reuse previously converted segments. Older entries are evicted when the cache fills."
            bind:checked={form.draft.cache_enabled}
          />
          <NumberField
            id="conversion-cache-limit"
            label="Conversion cache limit"
            unit="GB"
            step={0.5}
            min={0.1}
            max={100}
            disabled={!form.draft.cache_enabled}
            bind:value={form.draft.cache_limit_gb}
            error={form.errors.cache_limit_gb}
          />
        </Field.Group>
      </SettingsRow>
    {/if}
  </SettingsFormBody>
</SettingsFrame>
