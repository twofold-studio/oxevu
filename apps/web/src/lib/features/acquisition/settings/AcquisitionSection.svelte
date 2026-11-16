<script lang="ts">
  import { onMount, onDestroy, untrack } from "svelte";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { AcquisitionConfig } from "$lib/api/acquisition-types";
  import { errorMessage } from "$lib/api/client";
  import { session } from "$lib/state/session.svelte";
  import { Autosave } from "$lib/features/settings/autosave";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import SettingsSection from "$lib/features/settings/SettingsSection.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import ServerFields from "./ServerFields.svelte";
  import IndexersSection from "./IndexersSection.svelte";
  import ProfilesSection from "./ProfilesSection.svelte";
  import PermissionsSection from "./PermissionsSection.svelte";
  import MigrationSection from "./MigrationSection.svelte";
  const resource = createResource(() => acquisitionApi.settings());
  let draft = $state<AcquisitionConfig | null>(null);let status = $state("Changes save automatically.");let error = $state<string | null>(null);
  const invalid = $derived(draft ? validate(draft) : null);
  function validate(c: AcquisitionConfig): string | null {
    if (!c.download_dir.startsWith("/")) return "Use an absolute download-folder path.";
    const numeric = [c.listen_port,c.max_active_downloads,c.download_limit_kib,c.upload_limit_kib,c.min_free_gb,c.storage_limit_gb,c.seed_ratio,c.seed_hours,c.search_interval_minutes];
    if (numeric.some((n) => !Number.isFinite(n) || n < 0)) return "Enter valid nonnegative numeric values.";
    if (c.max_active_downloads < 1 || c.max_active_downloads > 16 || c.listen_port > 65535 || c.storage_limit_gb < 1 || c.storage_limit_gb > 100000 || c.min_free_gb > 100000 || c.seed_ratio > 100 || c.seed_hours > 87600 || c.search_interval_minutes < 5 || c.search_interval_minutes > 1440) return "One or more values are outside the allowed range.";
    if ([c.download_limit_kib,c.upload_limit_kib].some((n) => (n > 0 && n < 16) || n > 1000000)) return "Speed limits must be zero or at least 16 KiB/s, up to 1,000,000.";
    return null;
  }
  const autosave = new Autosave<AcquisitionConfig>(async (value) => { await acquisitionApi.configure(value); }, (state, failure) => { error = state === "error" ? errorMessage(failure) : null; status = state === "saving" ? "Saving changes…" : state === "error" ? "Changes were not saved." : "Changes saved."; });
  onMount(async () => { if (!session.user?.is_admin) return;await resource.load();if (resource.value) { autosave.reset(resource.value.settings);draft = structuredClone($state.snapshot(resource.value.settings)); } });
  $effect(() => { const value = draft && !invalid ? $state.snapshot(draft) : null;untrack(() => autosave.update(value)); });
  onDestroy(() => { void autosave.flush(); });
</script>
{#if session.user?.is_admin}
<ResourceGate status={resource.status} error={resource.error} what="acquisition settings" onretry={resource.load}>
  {#if draft}
    <div class="flex flex-col gap-5">
      <SettingsSection icon={DownloadIcon} title="Downloads and imports" description="Embedded BitTorrent transfers, separate storage, queue bounds and seeding rules. Network or storage changes restart the engine and recheck managed files.">
        <ServerFields bind:draft />
        <p role="status" class="mt-5 text-sm text-muted-foreground">{#if error}<span class="text-destructive">{error}</span>{:else if invalid}{invalid}{:else}{status}{/if}</p>
      </SettingsSection>
      <IndexersSection />
      <ProfilesSection />
      <PermissionsSection />
      <MigrationSection />
    </div>
  {/if}
</ResourceGate>
{:else}<IndexersSection />{/if}
