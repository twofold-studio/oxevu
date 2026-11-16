<script lang="ts">
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import type { AcquisitionConfig } from "$lib/api/acquisition-types";
  import NumberField from "$lib/features/settings/playback/NumberField.svelte";
  import SwitchField from "$lib/features/settings/playback/SwitchField.svelte";
  import PathInput from "$lib/features/settings/libraries/PathInput.svelte";
  let { draft = $bindable() }: { draft: AcquisitionConfig } = $props();
  const numbers = [
    { key: "listen_port", label: "Incoming torrent port", min: 0, max: 65535, unit: undefined, description: "6882 avoids the existing qBittorrent port. Zero chooses an ephemeral port." },
    { key: "max_active_downloads", label: "Active downloads", min: 1, max: 16, unit: undefined, description: "Completed torrents can continue seeding." },
    { key: "download_limit_kib", label: "Download speed limit", min: 0, max: 1000000, unit: "KiB/s", description: "Zero is unlimited. Nonzero limits must be at least 16 KiB/s." },
    { key: "upload_limit_kib", label: "Upload speed limit", min: 0, max: 1000000, unit: "KiB/s", description: "Zero is unlimited. Nonzero limits must be at least 16 KiB/s." },
    { key: "storage_limit_gb", label: "Download storage ceiling", min: 1, max: 100000, unit: "GiB", description: "Transfers pause when sampled storage usage reaches the limit." },
    { key: "min_free_gb", label: "Keep free", min: 0, max: 100000, unit: "GiB", description: "Reserve room for playback and other services." },
    { key: "seed_ratio", label: "Seed ratio target", min: 0, max: 100, unit: undefined, description: "Zero disables this target. Enabled ratio and time targets must both be met." },
    { key: "seed_hours", label: "Seed time target", min: 0, max: 87600, unit: "hours", description: "Zero disables this target. Reaching targets pauses seeding without deleting files." },
    { key: "search_interval_minutes", label: "Missing-title search interval", min: 5, max: 1440, unit: "min", description: "Applies only to titles with automatic acquisition enabled." },
  ] as const;
</script>
<Field.Group>
  <SwitchField id="acq-enabled" label="Enable downloads" description="Run the embedded torrent engine. Direct traffic is allowed; no VPN is required." bind:checked={draft.enabled} />
  <Field.Field><Field.Label>Download folder</Field.Label><PathInput bind:value={draft.download_dir} label="Download folder" /><Field.Description>Keep downloads separate from library roots, source code and conversion caches.</Field.Description></Field.Field>
  <Field.Field><Field.Label for="acq-interface">Bind to interface</Field.Label><Input id="acq-interface" value={draft.bind_interface ?? ""} oninput={(e) => draft.bind_interface = e.currentTarget.value.trim() || null} placeholder="Optional, e.g. wg0" maxlength={15} /><Field.Description>Blank uses normal routes. Binding alone does not establish a VPN kill switch.</Field.Description></Field.Field>
  <SwitchField id="acq-dht" label="Public peer discovery" description="Use DHT for public torrents. Private torrents keep their tracker restrictions." bind:checked={draft.dht} />
  <div class="grid gap-5 sm:grid-cols-2">{#each numbers as field (field.key)}<NumberField id={`acq-${field.key}`} label={field.label} description={field.description} unit={field.unit} min={field.min} max={field.max} step={field.key === "seed_ratio" ? 0.1 : 1} bind:value={draft[field.key]} />{/each}</div>
  <SwitchField id="acq-imports" label="Allow library imports" description="Permit reviewed imports and configured per-title destination rules to write completed media into libraries." bind:checked={draft.import_enabled} />
  <SwitchField id="acq-copy" label="Cross-filesystem copy fallback" description="If a hardlink is impossible across filesystems, copy and verify the file. The seeding source remains." bind:checked={draft.copy_fallback} />
</Field.Group>
