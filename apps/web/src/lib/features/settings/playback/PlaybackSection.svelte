<script lang="ts">
  import { onMount, onDestroy, untrack } from "svelte";
  import CpuIcon from "@lucide/svelte/icons/cpu";
  import GaugeIcon from "@lucide/svelte/icons/gauge";
  import HistoryIcon from "@lucide/svelte/icons/history";
  import ActivityIcon from "@lucide/svelte/icons/activity";
  import { settingsApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import type { Settings, SettingsEnvelope } from "$lib/api/admin-types";
  import { Autosave } from "../autosave";
  import ResourceGate from "../ResourceGate.svelte";
  import SettingsSection from "../SettingsSection.svelte";
  import { createResource } from "../resource.svelte";
  import CapabilityPanel from "./CapabilityPanel.svelte";
  import EncoderFields from "./EncoderFields.svelte";
  import LimitFields from "./LimitFields.svelte";
  import ResumeFields from "./ResumeFields.svelte";
  import { hasSettingsErrors, settingsEqual, validateSettings } from "./settings-form";

  const envelope = createResource(() => settingsApi.get());
  let draft = $state<Settings | null>(null);
  let saved = $state(false);
  let pending = $state(false);
  let failure = $state<string | null>(null);

  const capabilities = $derived(envelope.value?.capabilities ?? null);
  const errors = $derived(draft ? validateSettings(draft, capabilities) : {});
  const dirty = $derived(Boolean(draft && envelope.value && !settingsEqual(draft, envelope.value.settings)));
  const shown = $derived(dirty ? errors : {});

  const autosave = new Autosave<Settings>(async (snapshot) => {
    const next: SettingsEnvelope = await settingsApi.save(snapshot);
    envelope.value = next;
  }, (state, error) => {
    pending = state === "saving";
    saved = state === "saved";
    failure = state === "error" ? errorMessage(error) : null;
  });

  onMount(async () => {
    await envelope.load();
    if (envelope.value) {
      autosave.reset(envelope.value.settings);
      draft = { ...envelope.value.settings };
    }
  });
  $effect(() => {
    const value = draft && !hasSettingsErrors(errors) ? $state.snapshot(draft) : null;
    untrack(() => autosave.update(value));
  });
  onDestroy(() => { void autosave.flush(); });
</script>

<ResourceGate status={envelope.status} error={envelope.error} what="playback settings" onretry={envelope.load}>
  {#if draft && capabilities}
    <form class="flex flex-col gap-4" onsubmit={(event) => event.preventDefault()} novalidate>
      <SettingsSection
        icon={ActivityIcon}
        title="Server capabilities"
        description="Detected when the server starts. Conversion needs FFmpeg; NVIDIA acceleration needs a working NVENC encoder."
      >
        <CapabilityPanel {capabilities} />
      </SettingsSection>

      <SettingsSection
        icon={CpuIcon}
        title="Conversion"
        description="Used when a browser can't play the original file, or a viewer chooses Convert. Output is H.264 video with AAC audio."
      >
        <EncoderFields bind:draft {capabilities} errors={shown} />
      </SettingsSection>

      <SettingsSection
        icon={GaugeIcon}
        title="Limits"
        description="Bound how much work and disk space conversion can use. Active streams keep the settings they started with."
      >
        <LimitFields bind:draft errors={shown} />
      </SettingsSection>

      <SettingsSection
        icon={HistoryIcon}
        title="Resume"
        description="When a partly watched title offers to continue, and when it counts as watched."
      >
        <ResumeFields bind:draft errors={shown} />
      </SettingsSection>

      <p class="text-sm text-muted-foreground" role="status" aria-live="polite">
        {#if failure}<span class="text-destructive">Not saved: {failure}. Change a field to retry.</span>
        {:else if hasSettingsErrors(errors)}Fix the highlighted fields to save.
        {:else if pending || dirty}Saving changes…
        {:else if saved}Changes saved. They apply to new streams.
        {:else}Changes save automatically.{/if}
      </p>
    </form>
  {/if}
</ResourceGate>
