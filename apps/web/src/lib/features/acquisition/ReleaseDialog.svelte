<script lang="ts">
  import { onMount } from "svelte";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import * as Select from "$lib/components/ui/select";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Badge } from "$lib/components/ui/badge";
  import { Spinner } from "$lib/components/ui/spinner";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { DiscoverTitle, ProfileRecord, Release } from "$lib/api/acquisition-types";
  import { errorMessage } from "$lib/api/client";
  import { bytes } from "./format";
  import { href } from "$lib/state/router.svelte";
  let { open = $bindable(), title, season, episode, profileId = "default" }: { open: boolean; title: DiscoverTitle; season?: number; episode?: number; profileId?: string } = $props();
  let releases = $state<Release[]>([]);let profiles = $state<ProfileRecord[]>([]);let profile = $state("default");
  let loading = $state(false);let pending = $state<string | null>(null);let error = $state<string | null>(null);let warnings = $state<string[]>([]);let selected = $state<string | null>(null);let override = $state(false);let queued = $state(false);
  const options = $derived(profiles.map((p) => ({ value: p.id, label: p.profile.name })));
  onMount(async () => { profile = profileId; try { profiles = (await acquisitionApi.profiles()).profiles; } catch (e) { error = errorMessage(e); } });
  async function search() {
    loading = true;error = null;selected = null;override = false;
    try { const result = await acquisitionApi.releases(title.id, { season, episode, profile_id: profile }); releases = result.releases; warnings = result.errors.map((e) => e.error); }
    catch (e) { error = errorMessage(e); } finally { loading = false; }
  }
  async function download(release: Release) {
    pending = release.id;error = null;
    try { await acquisitionApi.download({ release_id: release.id, profile_id: profile, override_profile: override, only_wanted: false }); queued = true; }
    catch (e) { error = errorMessage(e); } finally { pending = null; }
  }
</script>
<Dialog.Root bind:open><Dialog.Content size="wide">
  <Dialog.Header><Dialog.Title>Find a release · {title.title}</Dialog.Title><Dialog.Description>Review the source, size and profile checks before starting a transfer. Automatic upgrades are off.</Dialog.Description></Dialog.Header>
  {#if queued}<p role="status">Download queued.</p><Dialog.Footer><Button href={href.activity()}>Open activity</Button></Dialog.Footer>
  {:else}
    <div class="flex flex-wrap items-end gap-3"><Field.Field class="min-w-40 flex-1"><Field.Label for="release-profile">Acquisition profile</Field.Label><Select.Root type="single" items={options} bind:value={profile}><Select.Trigger id="release-profile" class="w-full"><Select.Value placeholder="Choose profile" /></Select.Trigger><Select.Content><Select.Group>{#each options as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content></Select.Root></Field.Field><Button onclick={search} disabled={loading}>{#if loading}<Spinner data-icon="inline-start" />{/if}Search indexers</Button></div>
    {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
    {#each warnings as warning, index (`${index}:${warning}`)}<p class="text-sm text-muted-foreground">{warning}</p>{/each}
    {#if releases.length}
      <ul class="divide-y rounded-lg border">
        {#each releases as release (release.id)}
          <li class="flex flex-col gap-2 p-3">
            <div class="flex flex-wrap items-start justify-between gap-3"><div class="min-w-0 flex-1"><p class="text-sm font-medium break-words">{release.title}</p><p class="mt-1 text-xs text-muted-foreground">{release.indexer_name} · {bytes(release.size)} · {release.seeders ?? "Unknown"} seeders · {release.height ? `${release.height}p` : "Resolution unknown"} · {release.codec ?? "Codec unknown"}</p></div><Badge variant={release.accepted ? "secondary" : "outline"}>{release.accepted ? "Meets profile" : "Review"}</Badge></div>
            {#if !release.identity_match}<p class="text-xs text-muted-foreground">Title identity needs your review.</p>{/if}
            {#if release.reasons.length}<p class="text-xs text-muted-foreground">{release.reasons.join(" · ")}</p>{/if}
            {#if selected === release.id && !release.accepted}<Field.Field orientation="horizontal"><Checkbox id="release-override" bind:checked={override} /><Field.Label for="release-override">Use this release despite the profile warnings</Field.Label></Field.Field>{/if}
            <div class="flex justify-end"><Button size="sm" variant="outline" disabled={pending !== null} onclick={() => { if (!release.accepted && selected !== release.id) { selected = release.id; override = false; } else if (release.accepted || override) void download(release); }}><DownloadIcon data-icon="inline-start" />{pending === release.id ? "Queuing…" : selected === release.id && !release.accepted ? "Confirm download" : "Download"}</Button></div>
          </li>
        {/each}
      </ul>
    {:else if !loading}<p class="text-sm text-muted-foreground">Choose a profile, then search your configured indexers.</p>{/if}
  {/if}
</Dialog.Content></Dialog.Root>
