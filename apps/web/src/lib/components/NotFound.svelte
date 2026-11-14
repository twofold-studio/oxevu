<script lang="ts">
  import CompassIcon from "@lucide/svelte/icons/compass";
  import * as Empty from "$lib/components/ui/empty";
  import { Button } from "$lib/components/ui/button";
  import { href } from "$lib/state/router.svelte";
  import Page from "./Page.svelte";

  let {
    title = "Page not found",
    description = "This address doesn't match anything in Oxevu.",
  }: { title?: string; description?: string } = $props();
</script>

<Page>
  <Empty.Root class="border">
    <Empty.Header>
      <Empty.Media variant="icon"><CompassIcon /></Empty.Media>
      <Empty.Title>{title}</Empty.Title>
      <Empty.Description>{description}</Empty.Description>
    </Empty.Header>
    <Empty.Content>
      <Button href={href.home()} variant="outline">Go home</Button>
    </Empty.Content>
  </Empty.Root>
</Page>
