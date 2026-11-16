<script lang="ts">
  import { onMount } from "svelte";
  import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import * as Empty from "$lib/components/ui/empty";
  import { Button } from "$lib/components/ui/button";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import SettingsFrame from "$lib/features/settings/layout/SettingsFrame.svelte";
  import { SETTINGS_GROUP } from "$lib/features/settings/layout/group";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { AcquisitionProfile, ProfileRecord } from "$lib/api/acquisition-types";
  import ProfileDialog from "./ProfileDialog.svelte";

  const resource = createResource(async () => (await acquisitionApi.profiles()).profiles);
  let open = $state(false);
  let opening = $state(0);
  let selected = $state<ProfileRecord | null>(null);

  onMount(() => {
    void resource.load();
  });

  function edit(record: ProfileRecord | null) {
    selected = record;
    opening++;
    open = true;
  }

  function summary(profile: AcquisitionProfile) {
    const parts = [
      `${profile.min_height}–${profile.max_height}p`,
      profile.max_size_mb === 0 ? "No maximum size" : `up to ${Number((profile.max_size_mb / 1024).toFixed(2))} GiB`,
      `${profile.min_seeders}+ seeders`,
    ];
    if (profile.codecs.length) parts.push(profile.codecs.join("/"));
    if (profile.allow_hdr) parts.push("HDR allowed");
    return parts.join(" · ");
  }
</script>

<SettingsFrame
  page="sources/profiles"
  description="Bound resolution, size, codecs, sources and seeders for downloads. Every search result explains its profile checks. Playback conversion quality is separate."
>
  {#snippet actions()}
    {#if resource.status === "ready"}
      <Button size="sm" onclick={() => edit(null)}><PlusIcon data-icon="inline-start" />Add profile</Button>
    {/if}
  {/snippet}

  <ResourceGate status={resource.status} error={resource.error} what="release profiles" onretry={resource.load}>
    {#if !resource.value?.length}
      <Empty.Root class="border">
        <Empty.Header>
          <Empty.Media variant="icon"><SlidersHorizontalIcon /></Empty.Media>
          <Empty.Title>No release profiles</Empty.Title>
          <Empty.Description>Add a profile to choose which releases are acceptable.</Empty.Description>
        </Empty.Header>
      </Empty.Root>
    {:else}
      <ul class={SETTINGS_GROUP}>
        {#each resource.value as record (record.id)}
          <li class="flex items-center justify-between gap-3 px-5 py-4">
            <div class="flex min-w-0 flex-col gap-1">
              <p class="truncate text-sm font-medium">{record.profile.name}</p>
              <p class="text-xs text-muted-foreground">{summary(record.profile)}</p>
            </div>
            <Button size="sm" variant="ghost" onclick={() => edit(record)} aria-label={`Edit ${record.profile.name}`}>Edit</Button>
          </li>
        {/each}
      </ul>
    {/if}
  </ResourceGate>
</SettingsFrame>

{#key opening}
  {#if open}
    <ProfileDialog bind:open record={selected} onsaved={() => void resource.load()} />
  {/if}
{/key}
