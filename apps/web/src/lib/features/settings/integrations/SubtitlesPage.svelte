<script lang="ts">
  import CaptionsIcon from "@lucide/svelte/icons/captions";
  import { cn } from "$lib/utils";
  import ResourceGate from "../ResourceGate.svelte";
  import SettingsFrame from "../layout/SettingsFrame.svelte";
  import SettingsRow from "../layout/SettingsRow.svelte";
  import { SETTINGS_GROUP } from "../layout/group";
  import ProviderFields from "./ProviderFields.svelte";
  import { createIntegrations } from "./integrations.svelte";

  const integrations = createIntegrations();
  const resource = integrations.resource;
</script>

<SettingsFrame
  page="media/subtitles"
  description="Subtitle search from the player. Downloaded subtitles go into Oxevu's data folder; media folders are not written."
>
  <ResourceGate status={resource.status} error={resource.error} what="subtitle providers" onretry={resource.load}>
    {#if resource.value}
      <div class="flex flex-col gap-4">
        <div class={SETTINGS_GROUP}>
          <SettingsRow
            icon={CaptionsIcon}
            title="OpenSubtitles"
            description="Find and download subtitles. Requires a username, password and application API key."
          >
            <div class="flex flex-col gap-4">
              <ProviderFields
                provider="opensubtitles"
                value={resource.value.integrations.opensubtitles}
                update={(patch) => integrations.update("opensubtitles", patch)}
              />
              <a
                class="text-sm underline underline-offset-4"
                href="https://www.opensubtitles.com/consumers"
                target="_blank"
                rel="noreferrer">Get credentials from OpenSubtitles</a
              >
            </div>
          </SettingsRow>
        </div>
        <p class={cn("text-sm", integrations.failure ? "text-destructive" : "text-muted-foreground")} role="status" aria-live="polite">
          {integrations.status}
        </p>
      </div>
    {/if}
  </ResourceGate>
</SettingsFrame>
