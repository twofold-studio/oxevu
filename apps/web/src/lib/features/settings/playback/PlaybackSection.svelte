<script lang="ts">
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import CpuIcon from "@lucide/svelte/icons/cpu";
  import GaugeIcon from "@lucide/svelte/icons/gauge";
  import HistoryIcon from "@lucide/svelte/icons/history";
  import ActivityIcon from "@lucide/svelte/icons/activity";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { settingsApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import type { Settings, SettingsEnvelope } from "$lib/api/admin-types";
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
  let submitted = $state(false);
  let pending = $state(false);
  let failure = $state<string | null>(null);

  const capabilities = $derived(envelope.value?.capabilities ?? null);
  const errors = $derived(draft ? validateSettings(draft, capabilities) : {});
  const shown = $derived(submitted ? errors : {});
  const dirty = $derived(Boolean(draft && envelope.value && !settingsEqual(draft, envelope.value.settings)));

  function adopt(next: SettingsEnvelope) {
    envelope.value = next;
    draft = { ...next.settings };
  }

  onMount(async () => {
    await envelope.load();
    if (envelope.value) draft = { ...envelope.value.settings };
  });

  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (!draft || pending) return;
    submitted = true;
    failure = null;
    if (hasSettingsErrors(errors)) return;
    pending = true;
    try {
      adopt(await settingsApi.save(draft));
      submitted = false;
      toast.success("Playback settings saved. They apply to new streams.");
    } catch (error) {
      failure = errorMessage(error);
    } finally {
      pending = false;
    }
  }

  function discard() {
    if (envelope.value) draft = { ...envelope.value.settings };
    submitted = false;
    failure = null;
  }
</script>

<ResourceGate status={envelope.status} error={envelope.error} what="playback settings" onretry={envelope.load}>
  {#if draft && capabilities}
    <form class="flex flex-col gap-4" onsubmit={save} novalidate>
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

      {#if dirty || failure}
        <div
          class="sticky bottom-20 z-10 flex flex-wrap items-center justify-between gap-3 rounded-xl border bg-popover px-4 py-3 shadow-lg md:bottom-4"
          role="region"
          aria-label="Unsaved changes"
        >
          <p class="text-sm" role={failure ? "alert" : undefined}>
            {#if failure}
              <span class="text-destructive">Not saved: {failure}</span>
            {:else if submitted && hasSettingsErrors(errors)}
              <span class="text-destructive">Fix the highlighted fields.</span>
            {:else}
              Unsaved changes
            {/if}
          </p>
          <div class="flex gap-2">
            <Button variant="outline" onclick={discard} disabled={pending}>Discard</Button>
            <Button type="submit" disabled={pending}>
              {#if pending}<Spinner data-icon="inline-start" />{/if}
              Save
            </Button>
          </div>
        </div>
      {/if}
    </form>
  {/if}
</ResourceGate>
