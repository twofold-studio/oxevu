<script lang="ts">
  import ServerOffIcon from "@lucide/svelte/icons/server-off";
  import * as Empty from "$lib/components/ui/empty";
  import { Button } from "$lib/components/ui/button";

  let { message, onretry }: { message: string | null; onretry: () => void } = $props();
</script>

<div class="grid min-h-dvh place-items-center p-5">
  <Empty.Root class="max-w-md border">
    <Empty.Header>
      <Empty.Media variant="icon"><ServerOffIcon /></Empty.Media>
      <Empty.Title>Can't reach the Oxevu server</Empty.Title>
      <Empty.Description>{message ?? "The server didn't answer."}</Empty.Description>
    </Empty.Header>
    <Empty.Content>
      <Button onclick={onretry}>Try again</Button>
    </Empty.Content>
  </Empty.Root>
</div>
