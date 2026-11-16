<script lang="ts">
  import { onMount } from "svelte";
  import CheckIcon from "@lucide/svelte/icons/check";
  import XIcon from "@lucide/svelte/icons/x";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import InboxIcon from "@lucide/svelte/icons/inbox";
  import * as Empty from "$lib/components/ui/empty";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import { Input } from "$lib/components/ui/input";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import Artwork from "$lib/components/Artwork.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import { errorMessage } from "$lib/api/client";
  import type { MediaRequest } from "$lib/api/acquisition-types";
  import { session } from "$lib/state/session.svelte";
  import { href } from "$lib/state/router.svelte";
  import ResourceGate from "$lib/features/settings/ResourceGate.svelte";
  import { createResource } from "$lib/features/settings/resource.svelte";
  import { scopeLabel } from "./format";
  const resource = createResource(async () => (await acquisitionApi.requests()).requests);
  let filter = $state("all");let search = $state("");let pending = $state<string | null>(null);let error = $state<string | null>(null);
  let declining = $state<MediaRequest | null>(null);let declineOpen = $state(false);let note = $state("");
  let cancelling = $state<MediaRequest | null>(null);let cancelOpen = $state(false);
  const manage = $derived(session.user?.is_admin || session.user?.acquisition?.manage_requests);
  const shown = $derived((resource.value ?? []).filter((r) => (filter === "all" || r.status === filter) && `${r.title.title} ${r.username}`.toLowerCase().includes(search.toLowerCase())));
  onMount(() => { void resource.load(); const timer = setInterval(() => { if (document.visibilityState === "visible") void resource.load(); }, 15000); return () => clearInterval(timer); });
  async function decide(request: MediaRequest, status: string, reason = "") {
    pending = request.id;error = null;
    try { await acquisitionApi.decide(request.id, status, reason); declineOpen = false; cancelOpen = false; await resource.load(); }
    catch (e) { error = errorMessage(e); } finally { pending = null; }
  }
</script>
<Page class="max-w-6xl">
  <PageHeader title="Requests" description={manage ? "Approve household requests and follow their availability." : "Your requested movies and episodes."}>
    {#snippet actions()}<Button variant="outline" onclick={resource.load} aria-label="Refresh requests"><RotateCwIcon /></Button><Button href={href.discover()}>Find a title</Button>{/snippet}
  </PageHeader>
  <div class="mb-5 flex flex-wrap items-center gap-3">
    <ToggleGroup.Root type="single" variant="outline" bind:value={() => filter, (v) => v && (filter = v)} aria-label="Request status">
      {#each ["all", "pending", "approved", "fulfilled"] as value (value)}<ToggleGroup.Item {value} class="capitalize">{value === "fulfilled" ? "Available" : value}</ToggleGroup.Item>{/each}
    </ToggleGroup.Root>
    <Input aria-label="Filter requests by title or requester" placeholder="Filter requests" bind:value={search} class="max-w-64" />
  </div>
  {#if error}<p role="alert" class="mb-4 text-sm text-destructive">{error}</p>{/if}
  <ResourceGate status={resource.status} error={resource.error} what="requests" onretry={resource.load}>
    {#if shown.length === 0}
      <Empty.Root class="border"><Empty.Header><Empty.Media variant="icon"><InboxIcon /></Empty.Media><Empty.Title>No requests here</Empty.Title><Empty.Description>Search Discover to request a movie, a season, or individual episodes.</Empty.Description></Empty.Header></Empty.Root>
    {:else}
      <ul class="divide-y rounded-xl border bg-card">
        {#each shown as request (request.id)}
          <li class="flex flex-wrap items-center gap-4 p-4">
            <Artwork src={request.title.poster_url} alt="" class="hidden aspect-[2/3] w-14 shrink-0 rounded border sm:block" />
            <div class="flex min-w-0 flex-1 flex-col gap-1.5">
              <a class="font-medium hover:underline" href={href.discoverTitle(request.title.kind, request.title.provider_id)}>{request.title.title} {request.title.year ? `(${request.title.year})` : ""}</a>
              <p class="text-xs text-muted-foreground">{scopeLabel(request.scope)} · {request.username} · {new Date(request.created_at * 1000).toLocaleDateString()}</p>
              <div class="flex flex-wrap items-center gap-2"><Badge variant={request.status === "pending" ? "outline" : "secondary"} class="capitalize">{request.status === "fulfilled" ? "Available" : request.status}</Badge>{#if request.availability.offline}<span class="text-xs text-muted-foreground">Storage offline</span>{/if}</div>
              {#if request.status === "approved"}<p class="text-xs text-muted-foreground">{request.stage}</p>{/if}
              {#if request.note}<p class="text-sm text-muted-foreground">{request.note}</p>{/if}
            </div>
            <div class="flex flex-wrap items-center gap-2">
              {#if manage && request.status === "pending"}
                <Button size="sm" disabled={pending !== null} onclick={() => decide(request, "approved")}><CheckIcon data-icon="inline-start" />Approve</Button>
                <Button variant="outline" size="sm" onclick={() => { declining = request; note = ""; declineOpen = true; }}><XIcon data-icon="inline-start" />Decline</Button>
              {/if}
              {#if request.availability.items[0]}<Button size="sm" variant="outline" href={href.item(request.availability.items[0].id)}>Open library</Button>{/if}
              {#if ["pending", "approved", "fulfilled"].includes(request.status) && (manage || request.user_id === session.user?.id)}
                <Button size="sm" variant="ghost" onclick={() => { cancelling = request; cancelOpen = true; }}>Cancel request</Button>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </ResourceGate>
</Page>
<Dialog.Root bind:open={declineOpen}><Dialog.Content><Dialog.Header><Dialog.Title>Decline {declining?.title.title}</Dialog.Title><Dialog.Description>The requester can read your explanation.</Dialog.Description></Dialog.Header>
  <Field.Field><Field.Label for="decline-note">Reason</Field.Label><Textarea id="decline-note" bind:value={note} maxlength={2000} /></Field.Field>
  <Dialog.Footer><Button variant="outline" onclick={() => declineOpen = false}>Back</Button><Button disabled={pending !== null} onclick={() => declining && decide(declining, "declined", note)}>Decline request</Button></Dialog.Footer>
</Dialog.Content></Dialog.Root>
<AlertDialog.Root bind:open={cancelOpen}><AlertDialog.Content><AlertDialog.Header><AlertDialog.Title>Cancel this request?</AlertDialog.Title><AlertDialog.Description>This cancels your request for {cancelling?.title.title}. Other household requests and downloaded files are retained.</AlertDialog.Description></AlertDialog.Header><AlertDialog.Footer><AlertDialog.Cancel>Keep request</AlertDialog.Cancel><AlertDialog.Action onclick={() => cancelling && decide(cancelling, "cancelled")}>Cancel request</AlertDialog.Action></AlertDialog.Footer></AlertDialog.Content></AlertDialog.Root>
