<script lang="ts">
  import type { Snippet } from "svelte";
  import ShieldAlertIcon from "@lucide/svelte/icons/shield-alert";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import * as Alert from "$lib/components/ui/alert";
  import * as Empty from "$lib/components/ui/empty";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import type { ResourceStatus } from "./resource.svelte";

  let {
    status,
    error,
    what,
    onretry,
    children,
  }: { status: ResourceStatus; error: string | null; what: string; onretry: () => void; children: Snippet } =
    $props();
</script>

{#if status === "ready"}
  {@render children()}
{:else if status === "forbidden"}
  <Empty.Root class="border">
    <Empty.Header>
      <Empty.Media variant="icon"><ShieldAlertIcon /></Empty.Media>
      <Empty.Title>Administrator access needed</Empty.Title>
      <Empty.Description>
        Only administrators can open {what}. Ask one if you need something changed.
      </Empty.Description>
    </Empty.Header>
  </Empty.Root>
{:else if status === "error"}
  <Alert.Root variant="destructive">
    <Alert.Title>Couldn't load {what}</Alert.Title>
    <Alert.Description>{error}</Alert.Description>
    <Alert.Action>
      <Button variant="outline" size="sm" onclick={onretry}>
        <RotateCwIcon data-icon="inline-start" />
        Retry
      </Button>
    </Alert.Action>
  </Alert.Root>
{:else}
  <div class="divide-y rounded-xl border" aria-busy="true" aria-label={`Loading ${what}`}>
    {#each [0, 1, 2] as i (i)}
      <div class="flex items-center gap-4 px-5 py-4">
        <div class="flex flex-1 flex-col gap-2">
          <Skeleton class="h-4 w-40" />
          <Skeleton class="h-3 w-64" />
        </div>
        <Skeleton class="h-8 w-20" />
      </div>
    {/each}
  </div>
{/if}
