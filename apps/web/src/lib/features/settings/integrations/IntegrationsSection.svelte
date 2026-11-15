<script lang="ts">
  import { onMount } from "svelte";
  import PlugIcon from "@lucide/svelte/icons/plug";
  import { Switch } from "$lib/components/ui/switch";
  import * as Field from "$lib/components/ui/field";
  import { integrationsApi, type Provider, type ProviderUpdate } from "$lib/api/integrations";
  import { errorMessage } from "$lib/api/client";
  import { createResource } from "../resource.svelte";
  import ResourceGate from "../ResourceGate.svelte";
  import SettingsSection from "../SettingsSection.svelte";
  import ProviderFields from "./ProviderFields.svelte";

  const resource = createResource(integrationsApi.get);
  const providers: { id: Provider; title: string; description: string; link: string }[] = [
    { id: "tmdb", title: "TMDB", description: "Movie and series metadata, episode titles and artwork. Requires a free developer key or read access token.", link: "https://www.themoviedb.org/settings/api" },
    { id: "omdb", title: "OMDb", description: "Additional movie information and IMDb-linked metadata. Requires an OMDb API key.", link: "https://www.omdbapi.com/apikey.aspx" },
    { id: "opensubtitles", title: "OpenSubtitles", description: "Find and download subtitles into Oxevu's data folder. Requires a username, password and application API key.", link: "https://www.opensubtitles.com/consumers" },
  ];
  let status = $state("Changes save automatically when you finish editing a field.");
  let failure = $state(false);
  let queue = Promise.resolve();
  let revision = 0;

  function update(provider: Provider | "studio_images", patch: ProviderUpdate) {
    const current = ++revision;
    status = "Saving changes…";
    failure = false;
    queue = queue.then(async () => {
      try {
        resource.value = await integrationsApi.update({ [provider]: patch });
        if (revision === current) status = "Changes saved.";
      } catch (error) {
        if (revision === current) {
          status = `Not saved: ${errorMessage(error)}. Edit the field to retry.`;
          failure = true;
        }
      }
    });
  }
  onMount(() => { void resource.load(); });
</script>

<ResourceGate status={resource.status} error={resource.error} what="integrations" onretry={resource.load}>
  {#if resource.value}
    <div class="flex flex-col gap-4">
      <SettingsSection icon={PlugIcon} title="Studio images" description="Studio artwork without a personal API key.">
        <Field.Group>
          <Field.Field orientation="horizontal">
            <Switch id="studio-images" checked={resource.value.integrations.studio_images.enabled}
              onCheckedChange={(enabled) => update("studio_images", { enabled })} />
            <Field.Label for="studio-images">Enabled</Field.Label>
          </Field.Field>
          <Field.Description>Public artwork from <a class="underline" href="https://github.com/jellyfin/jellyfin-artwork" target="_blank" rel="noreferrer">Jellyfin Artwork</a>, licensed under CC BY-SA 4.0.</Field.Description>
        </Field.Group>
      </SettingsSection>
      {#each providers as provider (provider.id)}
        <SettingsSection icon={PlugIcon} title={provider.title} description={provider.description}>
          <div class="flex flex-col gap-4">
            <ProviderFields provider={provider.id} value={resource.value.integrations[provider.id]} update={(patch) => update(provider.id, patch)} />
            <a class="text-sm underline underline-offset-4" href={provider.link} target="_blank" rel="noreferrer">Get credentials from {provider.title}</a>
          </div>
        </SettingsSection>
      {/each}
      <p class={failure ? "text-sm text-destructive" : "text-sm text-muted-foreground"} role="status" aria-live="polite">{status}</p>
      <a href="https://www.themoviedb.org" target="_blank" rel="noreferrer"><img src="/providers/tmdb.svg" alt="TMDB" class="h-4 w-auto" /></a>
      <p class="text-xs text-muted-foreground">This product uses the TMDB API but is not endorsed or certified by TMDB. Provider requests share the title being searched with that service.</p>
    </div>
  {/if}
</ResourceGate>
