<script lang="ts">
  import ShieldAlertIcon from "@lucide/svelte/icons/shield-alert";
  import * as Empty from "$lib/components/ui/empty";
  import { Button } from "$lib/components/ui/button";
  import { href } from "$lib/state/router.svelte";
  import { settingsPageLabel, type SettingsPageId } from "$lib/state/settings-pages";

  // A settings URL opened without permission. Navigation never offers these
  // pages; this explains a bookmark or a role that changed.
  let { page }: { page: SettingsPageId } = $props();

  const label = $derived(settingsPageLabel(page));
  const who = $derived(
    page === "sources/indexers" ? "administrators and indexer managers" : "administrators",
  );
</script>

<Empty.Root class="border">
  <Empty.Header>
    <Empty.Media variant="icon"><ShieldAlertIcon /></Empty.Media>
    <Empty.Title>You can't open {label.group} · {label.page}</Empty.Title>
    <Empty.Description>Only {who} can change these settings. Ask an administrator if you need something changed.</Empty.Description>
  </Empty.Header>
  <Empty.Content>
    <Button href={href.settings()} variant="outline">Open your settings</Button>
  </Empty.Content>
</Empty.Root>
