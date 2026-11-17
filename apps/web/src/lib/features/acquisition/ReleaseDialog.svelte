<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import * as Select from "$lib/components/ui/select";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { DiscoverTitle, ProfileRecord, ReleaseSource } from "$lib/api/acquisition-types";
  import { errorMessage } from "$lib/api/client";
  import { href } from "$lib/state/router.svelte";
  import { ReleaseSearch } from "./release-search.svelte";
  import ReleaseSearchStatus from "./ReleaseSearchStatus.svelte";
  import IndexerStatusList from "./IndexerStatusList.svelte";
  import ReleaseRow from "./ReleaseRow.svelte";

  let { open = $bindable(), title, season, episode, profileId = "default", autoSearch = false, onlyWanted = false }: { open: boolean; title: DiscoverTitle; season?: number; episode?: number; profileId?: string; autoSearch?: boolean; onlyWanted?: boolean } = $props();
  let profiles = $state<ProfileRecord[]>([]);
  let sources = $state<ReleaseSource[]>([]);
  // svelte-ignore state_referenced_locally
  let profile = $state(profileId);
  let loadError = $state<string | null>(null);
  let loadingOptions = $state(true);
  const searcher = new ReleaseSearch((profile, signal) => acquisitionApi.releases(title.id, { season, episode, profile_id: profile }, signal));
  const searchState = $derived(searcher.state);
  const completed = $derived(searchState.kind === "results" || searchState.kind === "empty" ? searchState : searchState.kind === "searching" ? searchState.previous : null);
  const result = $derived(completed?.result ?? (searchState.kind === "failed" ? searchState.result : undefined));
  const stale = $derived(Boolean(completed && completed.profile !== profile));
  const busy = $derived(searchState.kind === "searching");
  const visibleReleases = $derived(result?.releases.filter((release) =>
    (season === undefined || release.season === season) &&
    (episode === undefined || release.episode === episode)) ?? []);
  const accepted = $derived(visibleReleases.filter((release) => release.accepted).length);
  const options = $derived(profiles.map((record) => ({ value: record.id, label: record.profile.name })));

  onMount(() => {
    void Promise.all([acquisitionApi.profiles(), acquisitionApi.releaseSources()])
      .then(([p,s]) => {
        profiles = p.profiles; sources = s.sources;
        if (autoSearch && profiles.length && sources.length) search();
      })
      .catch((error) => loadError = errorMessage(error)).finally(() => loadingOptions = false);
  });
  onDestroy(() => searcher.cancel());
  function search() { if (!busy) void searcher.search(profile, sources.length); }
  function changeOpen(value: boolean) { if (!value) searcher.cancel(); open = value; }
</script>

<Dialog.Root {open} onOpenChange={changeOpen}><Dialog.Content size="wide">
  <Dialog.Header>
    <Dialog.Title>Find a release · {title.title}</Dialog.Title>
    <Dialog.Description>
      {#if onlyWanted}Only missing episodes tracked for this series will be selected from the release.
      {:else if episode !== undefined}Searching Season {season}, Episode {episode}. Review the release contents before downloading.
      {:else if season !== undefined}Searching {season === 0 ? "Specials" : `Season ${season}`}.
      {:else}Choose a release to download. Your configured library import settings apply.{/if}
    </Dialog.Description>
  </Dialog.Header>
  <div class="flex flex-wrap items-end gap-3">
    <Field.Field class="min-w-40 flex-1"><Field.Label for="release-profile">Acquisition profile</Field.Label>
      <Select.Root type="single" items={options} bind:value={profile} disabled={busy}><Select.Trigger id="release-profile" class="w-full"><Select.Value placeholder="Choose profile" /></Select.Trigger><Select.Content><Select.Group>{#each options as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content></Select.Root>
    </Field.Field>
    <Button onclick={search} disabled={busy || !profiles.length || !sources.length}>Search indexers</Button>
  </div>
  {#if loadError}<p role="alert" class="text-sm text-destructive">{loadError}</p>{/if}
  {#if !loadingOptions && !loadError && sources.length === 0}<p class="text-sm text-muted-foreground">No enabled indexers. <a class="underline" href={href.settings("sources/indexers")}>Configure sources</a> before searching.</p>{/if}
  <ReleaseSearchStatus {searchState} oncancel={() => searcher.cancel()} onretry={search} />
  {#if result}<IndexerStatusList {result} disabled={busy} onretry={search} />{/if}
  {#if stale}<p role="status" class="text-sm text-muted-foreground">Profile changed — search again.</p>{/if}
  {#if busy && !result}
    <div class="flex flex-col gap-3" aria-hidden="true">{#each [1,2,3] as row (row)}<Skeleton class="h-24 w-full rounded-lg" />{/each}</div>
  {:else if searchState.kind === "idle"}
    <p class="text-sm text-muted-foreground">Choose a profile, then search your configured indexers.</p>
  {/if}
  {#if completed && !busy && accepted === 0}
    <div class="flex flex-col gap-3 rounded-lg border p-4">
      <p class="text-sm">{visibleReleases.length ? "No releases matched this profile. Review the alternatives below." : "No releases found for this selection."}</p>
      <div class="flex flex-wrap gap-2">
        <Button size="sm" variant="outline" onclick={search}>Search again</Button>
        <Button size="sm" variant="ghost" onclick={() => document.getElementById("release-profile")?.focus()}>Change profile</Button>
        <Button size="sm" variant="ghost" href={href.settings("sources/indexers")}>Open indexer settings</Button>
      </div>
    </div>
  {/if}
  {#if visibleReleases.length}
    <p class="text-xs text-muted-foreground">{accepted} meet profile · {visibleReleases.length - accepted} need review</p>
    <ul class={["divide-y rounded-lg border transition-opacity", (busy || stale) && "opacity-50"]} aria-busy={busy}>
      {#each visibleReleases as release (release.id)}<ReleaseRow {release} {profile} {onlyWanted} disabled={busy || stale} />{/each}
    </ul>
  {/if}
</Dialog.Content></Dialog.Root>
