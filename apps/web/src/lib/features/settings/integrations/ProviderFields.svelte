<script lang="ts">
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import type { Provider, ProviderStatus, ProviderUpdate } from "$lib/api/integrations";
  import { Switch } from "$lib/components/ui/switch";

  let { provider, value, update }: {
    provider: Provider;
    value: ProviderStatus;
    update: (patch: ProviderUpdate) => void;
  } = $props();

  function secret(event: Event, key: "api_key" | "password") {
    const input = event.currentTarget as HTMLInputElement;
    if (input.value.trim()) update({ [key]: input.value });
    input.value = "";
  }
</script>

<Field.Group>
  <Field.Field orientation="horizontal">
    <Switch id={`${provider}-enabled`} checked={value.enabled} onCheckedChange={(enabled) => update({ enabled })} />
    <Field.Label for={`${provider}-enabled`}>Enabled</Field.Label>
  </Field.Field>
  <p class="text-sm text-muted-foreground">{value.configured ? "Credentials configured" : "Credentials required before this provider can be used"}</p>
  <Field.Field>
    <Field.Label for={`${provider}-key`}>API key{provider === "tmdb" ? " or read access token" : ""}</Field.Label>
    <Input id={`${provider}-key`} type="password" autocomplete="off"
      placeholder={value.api_key_set ? "Configured · enter to replace" : "Enter API key"}
      onchange={(event) => secret(event, "api_key")} />
    <Field.Description>Saved when you finish editing. Stored on the server; never returned to this browser.</Field.Description>
  </Field.Field>
  {#if provider === "opensubtitles"}
    <Field.Field>
      <Field.Label for="opensubtitles-username">Username</Field.Label>
      <Input id="opensubtitles-username" autocomplete="off" value={value.username ?? ""}
        onchange={(event) => update({ username: event.currentTarget.value.trim() })} />
    </Field.Field>
    <Field.Field>
      <Field.Label for="opensubtitles-password">Password</Field.Label>
      <Input id="opensubtitles-password" type="password" autocomplete="new-password"
        placeholder={value.password_set ? "Configured · enter to replace" : "Enter password"}
        onchange={(event) => secret(event, "password")} />
      <Field.Description>Use an OpenSubtitles.com account. Its API also requires an application key.</Field.Description>
    </Field.Field>
  {/if}
  {#if value.api_key_set || value.password_set}
    <Button type="button" variant="outline" onclick={() => update({ clear_credentials: true })}>Remove credentials</Button>
  {/if}
</Field.Group>
