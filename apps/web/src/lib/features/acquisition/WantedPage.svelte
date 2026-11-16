<script lang="ts">
  import { onMount } from "svelte";
  import SearchIcon from "@lucide/svelte/icons/search";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import ListChecksIcon from "@lucide/svelte/icons/list-checks";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import * as Empty from "$lib/components/ui/empty";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import Artwork from "$lib/components/Artwork.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { MonitoredTitle, WantedUnit } from "$lib/api/acquisition-types";
  import { session } from "$lib/state/session.svelte";
  import { href } from "$lib/state/router.svelte";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import { unitLabel, missingLabel } from "./format";
  import MonitorDialog from "./MonitorDialog.svelte";
  import ReleaseDialog from "./ReleaseDialog.svelte";
  const resource = createResource(async () => (await acquisitionApi.monitored()).titles);
  let filter = $state("missing");let opening = $state(0);let manageOpen = $state(false);let releaseOpen = $state(false);
  let selected = $state<MonitoredTitle | null>(null);let unit = $state<WantedUnit | null>(null);
  const canDownload = $derived(session.user?.is_admin || session.user?.acquisition?.manage_downloads);
  onMount(() => { void resource.load(); });
  const shown = $derived((resource.value ?? []).filter((e) => filter === "all" || e.units.some((u) => u.wanted && !u.available)));
</script>
<Page class="max-w-6xl">
  <PageHeader title="Wanted" description="Manage movies, series and episode monitoring. Available files stay separate from download jobs.">
    {#snippet actions()}<Button variant="outline" onclick={resource.load} aria-label="Refresh monitored titles"><RotateCwIcon /></Button><Button href={href.discover()}>Find a title</Button>{/snippet}
  </PageHeader>
  <ToggleGroup.Root class="mb-5" type="single" variant="outline" bind:value={() => filter, (v) => v && (filter = v)} aria-label="Wanted titles filter"><ToggleGroup.Item value="missing">Missing</ToggleGroup.Item><ToggleGroup.Item value="all">All monitored</ToggleGroup.Item></ToggleGroup.Root>
  <ResourceGate status={resource.status} error={resource.error} what="monitored titles" onretry={resource.load}>
    {#if shown.length === 0}<Empty.Root class="border"><Empty.Header><Empty.Media variant="icon"><ListChecksIcon /></Empty.Media><Empty.Title>No titles in this view</Empty.Title><Empty.Description>Approved requests appear here. Configure a destination to import their downloads.</Empty.Description></Empty.Header></Empty.Root>
    {:else}
      <div class="flex flex-col gap-4">
        {#each shown as entry (entry.title.id)}
          <section class="overflow-hidden rounded-xl border bg-card">
            <div class="flex flex-wrap items-center gap-4 p-4">
              <Artwork src={entry.title.poster_url} alt="" class="hidden aspect-[2/3] w-14 shrink-0 rounded border sm:block" />
              <div class="flex min-w-0 flex-1 flex-col gap-1.5"><h2 class="font-medium"><a href={href.discoverTitle(entry.title.kind, entry.title.provider_id)} class="hover:underline">{entry.title.title}</a></h2><p class="text-xs text-muted-foreground">{entry.units.filter((u) => u.wanted && u.available).length} / {entry.units.filter((u) => u.wanted).length} wanted entries available</p><div class="flex flex-wrap gap-2"><Badge variant="outline">{!entry.monitor.enabled ? "Paused" : entry.monitor.automatic ? "Automatic" : "Manual search"}</Badge>{#if !entry.monitor.root}<span class="text-xs text-muted-foreground">Destination needed</span>{/if}</div></div>
              {#if session.user?.is_admin}<Button variant="outline" size="sm" onclick={() => { selected = entry; opening++; manageOpen = true; }}><SettingsIcon data-icon="inline-start" />Manage</Button>{/if}
              {#if canDownload}<Button variant="outline" size="sm" onclick={() => { selected = entry; unit = null; opening++; releaseOpen = true; }}><SearchIcon data-icon="inline-start" />Find releases</Button>{/if}
            </div>
            <details class="border-t"><summary class="cursor-pointer px-4 py-3 text-sm">{entry.title.kind === "tv" ? "Episodes and availability" : "Movie availability"}</summary>
              <ul class="divide-y border-t">
                {#each entry.units as episode (`${episode.season}:${episode.episode}`)}
                  <li class="flex flex-wrap items-center gap-3 px-4 py-3"><div class="min-w-0 flex-1"><p class="text-sm">{unitLabel(episode)}</p><p class="mt-1 text-xs text-muted-foreground">{episode.air_date ?? "Release date unknown"} · {missingLabel(episode)}</p></div>{#if episode.item_id}<Button size="sm" variant="ghost" href={href.item(episode.item_id)}>Open</Button>{:else if canDownload && episode.wanted}<Button size="sm" variant="ghost" aria-label={`Find ${unitLabel(episode)}`} onclick={() => { selected = entry; unit = episode; opening++; releaseOpen = true; }}><SearchIcon /></Button>{/if}</li>
                {/each}
              </ul>
            </details>
          </section>
        {/each}
      </div>
    {/if}
  </ResourceGate>
</Page>
{#if selected}{#key opening}
  {#if manageOpen}<MonitorDialog bind:open={manageOpen} entry={selected} onsaved={() => { void resource.load(); }} />
  {:else if releaseOpen}<ReleaseDialog bind:open={releaseOpen} title={selected.title} season={unit?.season ?? undefined} episode={unit?.episode ?? undefined} profileId={selected.monitor.profile_id} />{/if}
{/key}{/if}
