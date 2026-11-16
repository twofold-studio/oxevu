<script lang="ts">
  import EthernetPortIcon from "@lucide/svelte/icons/ethernet-port";
  import NetworkIcon from "@lucide/svelte/icons/network";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import { acquisitionDocument } from "$lib/features/settings/documents";
  import { createSettingsForm } from "$lib/features/settings/settings-form.svelte";
  import SettingsFrame from "$lib/features/settings/layout/SettingsFrame.svelte";
  import SettingsFormBody from "$lib/features/settings/layout/SettingsFormBody.svelte";
  import SettingsRow from "$lib/features/settings/layout/SettingsRow.svelte";
  import NumberField from "$lib/features/settings/playback/NumberField.svelte";
  import SwitchField from "$lib/features/settings/playback/SwitchField.svelte";
  import { validateAcquisition } from "./acquisition-form";

  const form = createSettingsForm(
    acquisitionDocument,
    ["bind_interface", "dht", "listen_port"],
    validateAcquisition,
  );
</script>

<SettingsFrame
  page="downloads/network"
  description="How the torrent engine reaches peers. Changing any of these restarts the engine."
>
  <SettingsFormBody {form} what="download settings">
    {#if form.draft}
      <SettingsRow
        icon={NetworkIcon}
        title="Routing"
        description="Binding alone does not establish a VPN kill switch."
      >
        <Field.Group>
          <Field.Field>
            <Field.Label for="acq-interface">Bind to interface</Field.Label>
            <Input
              id="acq-interface"
              value={form.draft.bind_interface ?? ""}
              oninput={(event) => {
                if (form.draft) form.draft.bind_interface = event.currentTarget.value.trim() || null;
              }}
              placeholder="Optional, e.g. wg0"
              maxlength={15}
            />
            <Field.Description>Blank uses normal routes.</Field.Description>
          </Field.Field>
          <SwitchField
            id="acq-dht"
            label="Public peer discovery"
            description="Use DHT for public torrents. Private torrents keep their tracker restrictions."
            bind:checked={form.draft.dht}
          />
        </Field.Group>
      </SettingsRow>
      <SettingsRow icon={EthernetPortIcon} title="Incoming port" description="Peers connect to this port.">
        <NumberField
          id="acq-listen_port"
          label="Incoming torrent port"
          description="6882 avoids the existing qBittorrent port. Zero chooses an ephemeral port."
          min={0}
          max={65535}
          bind:value={form.draft.listen_port}
          error={form.errors.listen_port}
        />
      </SettingsRow>
    {/if}
  </SettingsFormBody>
</SettingsFrame>
