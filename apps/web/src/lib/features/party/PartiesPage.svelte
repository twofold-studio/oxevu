<script lang="ts">
  import { onMount } from "svelte";
  import UsersRoundIcon from "@lucide/svelte/icons/users-round";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import CrownIcon from "@lucide/svelte/icons/crown";
  import * as Empty from "$lib/components/ui/empty";
  import { Button } from "$lib/components/ui/button";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import { partiesApi } from "$lib/api/endpoints";
  import { catalog } from "$lib/state/catalog.svelte";
  import { session } from "$lib/state/session.svelte";
  import { href } from "$lib/state/router.svelte";
  import { canCreateParties } from "$lib/state/access";
  import { plural } from "$lib/media/format";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import CreatePartyDialog from "./CreatePartyDialog.svelte";
  import { itemLabel } from "./item-label";

  const parties = createResource(async () => (await partiesApi.list()).parties);
  let createOpen = $state(false);
  let opening = $state(0);

  const canCreate = $derived(canCreateParties(session.user));

  onMount(() => {
    void catalog.ensure();
    void parties.load();
    const timer = setInterval(() => void parties.load(), 5000);
    return () => clearInterval(timer);
  });

  function create() {
    opening++;
    createOpen = true;
  }

  function hostName(party: { host_user_id: string; members: { user_id: string; username: string }[] }) {
    return party.members.find((m) => m.user_id === party.host_user_id)?.username ?? "Host";
  }
</script>

<Page class="max-w-5xl">
  <PageHeader
    title="Watch parties"
    description="Join a party, then browse the library. Anyone can start a title for everyone; volume and subtitles stay personal."
  >
    {#snippet actions()}
      <Button variant="outline" size="sm" onclick={parties.load} aria-label="Refresh parties">
        <RotateCwIcon />
      </Button>
      {#if canCreate}
        <Button size="sm" onclick={create}>
          <PlusIcon data-icon="inline-start" />
          Start a party
        </Button>
      {/if}
    {/snippet}
  </PageHeader>

  <ResourceGate status={parties.status} error={parties.error} what="watch parties" onretry={parties.load}>
    {#if (parties.value ?? []).length === 0}
      <Empty.Root class="border">
        <Empty.Header>
          <Empty.Media variant="icon"><UsersRoundIcon /></Empty.Media>
          <Empty.Title>No parties right now</Empty.Title>
          <Empty.Description>
            {canCreate ? "Start one and share its link." : "Parties you can join appear here when someone starts one."}
          </Empty.Description>
        </Empty.Header>
      </Empty.Root>
    {:else}
      <ul class="divide-y rounded-xl border bg-card">
        {#each parties.value ?? [] as party (party.id)}
          {@const item = party.item_id ? catalog.find(party.item_id) : null}
          <li class="flex flex-wrap items-center gap-x-4 gap-y-2 px-5 py-4">
            <div class="flex min-w-0 flex-1 flex-col gap-1">
              <span class="truncate font-medium">{party.name}</span>
              <span class="truncate text-xs text-muted-foreground">
                {item ? itemLabel(item) : "Choosing something to watch"}{party.item_id ? ` · ${party.paused ? "Paused" : "Playing"}` : ""}
              </span>
              <span class="flex items-center gap-1.5 text-xs text-muted-foreground">
                <CrownIcon class="size-3.5" />{hostName(party)} · {plural(party.members.length, "member")}
              </span>
            </div>
            <Button href={href.party(party.id)} variant="outline">Join</Button>
          </li>
        {/each}
      </ul>
    {/if}
  </ResourceGate>
</Page>

{#key opening}
  <CreatePartyDialog bind:open={createOpen} />
{/key}
