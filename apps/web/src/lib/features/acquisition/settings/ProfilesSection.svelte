<script lang="ts">
  import { onMount } from "svelte";
  import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import { Button } from "$lib/components/ui/button";
  import SettingsSection from "$lib/features/settings/SettingsSection.svelte";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { ProfileRecord } from "$lib/api/acquisition-types";
  import ProfileDialog from "./ProfileDialog.svelte";
  const resource = createResource(async () => (await acquisitionApi.profiles()).profiles);
  let open = $state(false);let opening = $state(0);let selected = $state<ProfileRecord | null>(null);
  onMount(() => { void resource.load(); });
</script>
<SettingsSection icon={SlidersHorizontalIcon} title="Release profiles" description="Bound resolution, size, codecs, sources and seeders. Every search result explains its profile checks.">
  <ResourceGate status={resource.status} error={resource.error} what="release profiles" onretry={resource.load}>
    <div class="flex flex-col gap-3">
      {#each resource.value ?? [] as record (record.id)}<div class="flex items-center justify-between gap-3 border-b pb-3"><div><p class="text-sm font-medium">{record.profile.name}</p><p class="text-xs text-muted-foreground">{record.profile.min_height}–{record.profile.max_height}p · up to {Math.round(record.profile.max_size_mb / 1024)} GiB · {record.profile.min_seeders}+ seeders</p></div><Button size="sm" variant="ghost" onclick={() => { selected = record;opening++;open = true; }}>Edit</Button></div>{/each}
      <Button variant="outline" onclick={() => { selected = null;opening++;open = true; }}><PlusIcon data-icon="inline-start" />Add profile</Button>
    </div>
  </ResourceGate>
</SettingsSection>
{#key opening}{#if open}<ProfileDialog bind:open record={selected} onsaved={() => { void resource.load(); }} />{/if}{/key}
