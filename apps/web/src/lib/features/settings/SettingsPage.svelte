<script lang="ts">
  import { onMount } from "svelte";
  import Page from "$lib/components/Page.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import { session } from "$lib/state/session.svelte";
  import { href, type SettingsSection } from "$lib/state/router.svelte";
  import { settingsSectionsFor } from "$lib/state/access";
  import { cn } from "$lib/utils";
  import LibrariesSection from "./libraries/LibrariesSection.svelte";
  import PlaybackSection from "./playback/PlaybackSection.svelte";
  import UsersSection from "./users/UsersSection.svelte";
  import AccountSection from "./account/AccountSection.svelte";
  import IntegrationsSection from "./integrations/IntegrationsSection.svelte";
  import ResourceGate from "./ResourceGate.svelte";

  let { section }: { section: SettingsSection | null } = $props();

  const labels: Record<SettingsSection, string> = {
    libraries: "Libraries",
    playback: "Playback",
    integrations: "Integrations",
    users: "Users",
    account: "Account",
  };

  const allowed = $derived(settingsSectionsFor(session.user));
  const active = $derived(section ?? allowed[0]);

  onMount(() => {
    void session.refresh();
  });
</script>

<Page class="max-w-5xl">
  <PageHeader title="Settings" description="Server configuration and your account." />

  <nav aria-label="Settings sections" class="mb-6 border-b">
    <ul class="-mb-px flex gap-1 overflow-x-auto">
      {#each allowed as key (key)}
        <li>
          <a
            href={href.settings(key)}
            aria-current={active === key ? "page" : undefined}
            class={cn(
              "inline-flex h-10 items-center border-b-2 border-transparent px-3 text-sm text-muted-foreground transition-colors hover:text-foreground",
              active === key && "border-foreground font-medium text-foreground",
            )}>{labels[key]}</a
          >
        </li>
      {/each}
    </ul>
  </nav>

  {#key active}
    <div class="animate-rise">
      {#if !allowed.includes(active)}
        <!-- Admin-only section opened by a viewer: same explanation as a server 403. -->
        <ResourceGate status="forbidden" error={null} what={labels[active].toLowerCase()} onretry={() => undefined}>
          {""}
        </ResourceGate>
      {:else if active === "libraries"}
        <LibrariesSection />
      {:else if active === "playback"}
        <PlaybackSection />
      {:else if active === "integrations"}
        <IntegrationsSection />
      {:else if active === "users"}
        <UsersSection />
      {:else}
        <AccountSection />
      {/if}
    </div>
  {/key}
</Page>
