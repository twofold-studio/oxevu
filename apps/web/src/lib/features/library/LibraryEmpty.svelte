<script lang="ts">
  import type { Component } from "svelte";
  import * as Empty from "$lib/components/ui/empty";
  import { Button } from "$lib/components/ui/button";
  import { href } from "$lib/state/router.svelte";

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
  <Empty.Content>
    <Button href={href.libraries()} variant="outline">Manage libraries</Button>
  </Empty.Content>
</Empty.Root>
