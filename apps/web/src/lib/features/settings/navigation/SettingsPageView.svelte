<script lang="ts">
  import type { SettingsPageId } from "$lib/state/settings-pages";
  import LibrariesPage from "../libraries/LibrariesPage.svelte";
  import MetadataPage from "../integrations/MetadataPage.svelte";
  import SubtitlesPage from "../integrations/SubtitlesPage.svelte";
  import ConversionPage from "../playback/ConversionPage.svelte";
  import QualityPage from "../playback/QualityPage.svelte";
  import ScratchPage from "../playback/ScratchPage.svelte";
  import ResumePage from "../playback/ResumePage.svelte";
  import UsersPage from "../users/UsersPage.svelte";
  import ProfilePage from "../account/ProfilePage.svelte";
  import PasswordPage from "../account/PasswordPage.svelte";
  import PlayerPage from "../account/PlayerPage.svelte";

  // One permitted settings page. Acquisition pages load on demand, as the
  // rest of the acquisition interface does.
  let { page }: { page: SettingsPageId } = $props();

  const acquisition = {
    "downloads/storage": () => import("$lib/features/acquisition/settings/StoragePage.svelte"),
    "downloads/network": () => import("$lib/features/acquisition/settings/NetworkPage.svelte"),
    "downloads/queue": () => import("$lib/features/acquisition/settings/QueuePage.svelte"),
    "downloads/seeding": () => import("$lib/features/acquisition/settings/SeedingPage.svelte"),
    "downloads/imports": () => import("$lib/features/acquisition/settings/ImportsPage.svelte"),
    "sources/indexers": () => import("$lib/features/acquisition/settings/IndexersPage.svelte"),
    "sources/profiles": () => import("$lib/features/acquisition/settings/ProfilesPage.svelte"),
    "sources/migration": () => import("$lib/features/acquisition/settings/MigrationPage.svelte"),
    "people/requests": () => import("$lib/features/acquisition/settings/RequestAccessPage.svelte"),
  } as const;
  const lazy = $derived(page in acquisition ? acquisition[page as keyof typeof acquisition] : null);
</script>

{#if lazy}
  {#await lazy() then module}<module.default />{/await}
{:else if page === "media/libraries"}
  <LibrariesPage />
{:else if page === "media/metadata"}
  <MetadataPage />
{:else if page === "media/subtitles"}
  <SubtitlesPage />
{:else if page === "playback/conversion"}
  <ConversionPage />
{:else if page === "playback/quality"}
  <QualityPage />
{:else if page === "playback/scratch"}
  <ScratchPage />
{:else if page === "playback/resume"}
  <ResumePage />
{:else if page === "people/users"}
  <UsersPage />
{:else if page === "account/password"}
  <PasswordPage />
{:else if page === "account/player"}
  <PlayerPage />
{:else}
  <ProfilePage />
{/if}
