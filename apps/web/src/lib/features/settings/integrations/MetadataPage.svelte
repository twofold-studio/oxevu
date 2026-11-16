<script lang="ts">
  import ImageIcon from "@lucide/svelte/icons/image";
  import DatabaseIcon from "@lucide/svelte/icons/database";
  import { Switch } from "$lib/components/ui/switch";
  import * as Field from "$lib/components/ui/field";
  import type { Provider } from "$lib/api/integrations";
  import { cn } from "$lib/utils";
  import ResourceGate from "../ResourceGate.svelte";
  import SettingsFrame from "../layout/SettingsFrame.svelte";
  import SettingsRow from "../layout/SettingsRow.svelte";
  import { SETTINGS_GROUP } from "../layout/group";
  import ProviderFields from "./ProviderFields.svelte";
  import { createIntegrations } from "./integrations.svelte";

  const integrations = createIntegrations();
  const resource = integrations.resource;
  const providers: { id: Provider; title: string; description: string; link: string }[] = [
    {
      id: "tmdb",
      title: "TMDB",
      description: "Movie and series metadata, episode titles and artwork. Requires a free developer key or read access token.",
      link: "https://www.themoviedb.org/settings/api",
    },
    {
      id: "omdb",
      title: "OMDb",
      description: "Additional movie information and IMDb-linked metadata. Requires an OMDb API key.",
      link: "https://www.omdbapi.com/apikey.aspx",
    },
  ];
</script>

<SettingsFrame
  page="media/metadata"
  description="Where titles, artwork and ratings come from when you identify media or browse Discover. Credentials are stored on the server and never returned to the browser."
>
  <ResourceGate status={resource.status} error={resource.error} what="metadata providers" onretry={resource.load}>
    {#if resource.value}
      <div class="flex flex-col gap-4">
        <div class={SETTINGS_GROUP}>
          {#each providers as provider (provider.id)}
            <SettingsRow icon={DatabaseIcon} title={provider.title} description={provider.description}>
              <div class="flex flex-col gap-4">
                <ProviderFields
                  provider={provider.id}
                  value={resource.value.integrations[provider.id]}
                  update={(patch) => integrations.update(provider.id, patch)}
                />
                <a class="text-sm underline underline-offset-4" href={provider.link} target="_blank" rel="noreferrer">
                  Get credentials from {provider.title}
                </a>
              </div>
            </SettingsRow>
          {/each}
          <SettingsRow icon={ImageIcon} title="Studio images" description="Studio artwork without a personal API key.">
            <Field.Group>
              <Field.Field orientation="horizontal">
                <Switch
                  id="studio-images"
                  checked={resource.value.integrations.studio_images.enabled}
                  onCheckedChange={(enabled) => integrations.update("studio_images", { enabled })}
                />
                <Field.Label for="studio-images">Enabled</Field.Label>
              </Field.Field>
              <Field.Description>
                Public artwork from <a
                  class="underline"
                  href="https://github.com/jellyfin/jellyfin-artwork"
                  target="_blank"
                  rel="noreferrer">Jellyfin Artwork</a
                >, licensed under CC BY-SA 4.0.
              </Field.Description>
            </Field.Group>
          </SettingsRow>
        </div>
        <p class={cn("text-sm", integrations.failure ? "text-destructive" : "text-muted-foreground")} role="status" aria-live="polite">
          {integrations.status}
        </p>
        <div class="flex flex-col gap-2">
          <a href="https://www.themoviedb.org" target="_blank" rel="noreferrer" class="w-fit">
            <img src="/providers/tmdb.svg" alt="TMDB" class="h-4 w-auto" />
          </a>
          <p class="text-xs text-muted-foreground">
            This product uses the TMDB API but is not endorsed or certified by TMDB. Provider requests share the title being
            searched with that service.
          </p>
        </div>
      </div>
    {/if}
  </ResourceGate>
</SettingsFrame>
