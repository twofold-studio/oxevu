<script lang="ts">
  import type { Component } from "svelte";
  import * as Empty from "$lib/components/ui/empty";
  import { Button } from "$lib/components/ui/button";
  import { href } from "$lib/state/router.svelte";
  import { session } from "$lib/state/session.svelte";

  let {
    icon: Icon,
    title,
    description,
  }: { icon: Component; title: string; description: string } = $props();
</script>

<Empty.Root class="border">
  <Empty.Header>
    <Empty.Media variant="icon"><Icon /></Empty.Media>
    <Empty.Title>{title}</Empty.Title>
    <Empty.Description>{description}</Empty.Description>
  </Empty.Header>
  <!-- Viewers can't change libraries; an empty catalog may mean no access was granted. -->
  {#if session.user?.is_admin}
    <Empty.Content>
      <Button href={href.libraries()} variant="outline">Manage libraries</Button>
    </Empty.Content>
  {:else}
    <Empty.Content>
      <p class="text-sm text-muted-foreground">If you expected titles here, ask an administrator for library access.</p>
    </Empty.Content>
  {/if}
</Empty.Root>
