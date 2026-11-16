<script lang="ts">
  import { onMount } from "svelte";
  import ImportIcon from "@lucide/svelte/icons/import";
  import * as Field from "$lib/components/ui/field";
  import * as Select from "$lib/components/ui/select";
  import { Input } from "$lib/components/ui/input";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Button } from "$lib/components/ui/button";
  import SettingsSection from "$lib/features/settings/SettingsSection.svelte";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { MigrationInput, MigrationPreview } from "$lib/api/acquisition-types";
  import { errorMessage } from "$lib/api/client";
  let source = $state<MigrationInput["source"]>("radarr");
  let records = $state<unknown>(null);let filename = $state("");let users = $state("{}");let mappings = $state("{}");
  let preview = $state<MigrationPreview | null>(null);let reviewed: MigrationInput | null = null;
  let pending = $state(false);let error = $state<string | null>(null);let results = $state<{ key: string; imported: boolean; error?: string }[]>([]);
  let profile = $state("default");let profiles = $state([{ value: "default", label: "1080p" }]);
  onMount(async () => { try { profiles = (await acquisitionApi.profiles()).profiles.map((p) => ({ value: p.id, label: p.profile.name })); } catch (e) { error = errorMessage(e); } });
  const sources = [{ value: "radarr", label: "Radarr movies" }, { value: "sonarr", label: "Sonarr series" }, { value: "seerr", label: "Seerr / Jellyseerr requests" }];
  function invalidate() { preview = null;reviewed = null;results = []; }
  async function choose(event: Event) {
    invalidate();error = null;records = null;const file = (event.target as HTMLInputElement).files?.[0];
    if (!file) return;
    try { if (file.size > 1500000) throw new Error("Use an export under 1.5 MB, with at most 1000 records per batch.");records = JSON.parse(await file.text());filename = file.name; }
    catch (e) { error = errorMessage(e); }
  }
  async function review() {
    pending = true;error = null;
    try { const input: MigrationInput = { source, records, users: JSON.parse(users), tmdb_ids: JSON.parse(mappings), profile_id: profile };preview = await acquisitionApi.previewMigration(input);reviewed = { ...input, token: preview.token }; }
    catch (e) { error = errorMessage(e); } finally { pending = false; }
  }
  async function apply() {
    if (!reviewed) return;pending = true;error = null;
    try { results = (await acquisitionApi.migrate(reviewed)).results;preview = null;reviewed = null; }
    catch (e) { error = errorMessage(e); } finally { pending = false; }
  }
</script>
<SettingsSection icon={ImportIcon} title="Migration" description="Review API JSON exports before importing. Existing services, accounts and media stay in place. Imported titles require a destination and explicit automation setup under Wanted.">
  <Field.Group>
    <Field.Field><Field.Label for="migration-source">Export format</Field.Label><Select.Root type="single" items={sources} value={source} onValueChange={(value) => { source = value as MigrationInput["source"];invalidate(); }}><Select.Trigger id="migration-source"><Select.Value /></Select.Trigger><Select.Content><Select.Group>{#each sources as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content></Select.Root></Field.Field>
    <Field.Field><Field.Label for="migration-file">JSON export</Field.Label><Input id="migration-file" type="file" accept=".json,application/json" onchange={choose} /><Field.Description>{filename || "Radarr /api/v3/movie, Sonarr /api/v3/series, or Seerr /api/v1/request results. Export each page when paginated."}</Field.Description></Field.Field>
    <Field.Field><Field.Label for="migration-profile">Acquisition profile for new monitors</Field.Label><Select.Root type="single" items={profiles} value={profile} onValueChange={(value) => { profile = value;invalidate(); }}><Select.Trigger id="migration-profile"><Select.Value /></Select.Trigger><Select.Content><Select.Group>{#each profiles as option (option.value)}<Select.Item value={option.value}>{option.label}</Select.Item>{/each}</Select.Group></Select.Content></Select.Root><Field.Description>Import batches separately when old quality profiles need different mappings.</Field.Description></Field.Field>
    <Field.Field><Field.Label for="migration-tmdb">TMDB ID overrides</Field.Label><Textarea id="migration-tmdb" bind:value={mappings} oninput={invalidate} placeholder={'{"oldRecordId": 94605}'} /><Field.Description>JSON mapping from the export record ID to a TMDB ID. Required when a Sonarr export only has TVDB IDs. Identify the series first; titles alone are not used to guess.</Field.Description></Field.Field>
    {#if source === "seerr"}<Field.Field><Field.Label for="migration-users">Request owners</Field.Label><Textarea id="migration-users" bind:value={users} oninput={invalidate} placeholder={'{"oldUserId": "oxevu-user-id"}'} /><Field.Description>Every requester must map to an existing Oxevu user ID. Passwords and accounts are not imported.</Field.Description></Field.Field>{/if}
    <Button variant="outline" disabled={pending || !records} onclick={review}>{pending ? "Working…" : "Preview migration"}</Button>
    {#if preview}<p class="text-sm text-muted-foreground">{preview.notice}</p><ul class="max-h-72 overflow-y-auto divide-y rounded-lg border">{#each preview.records as row (row.key)}<li class="px-3 py-2 text-sm"><p>{row.title}</p><p class="text-xs text-muted-foreground">{row.key} · TMDB {row.provider_id} · {row.owner ? `Owner ${row.owner} · ${row.status}` : row.enabled ? "Monitored, automation off" : "Not monitored"}</p></li>{/each}</ul><Button disabled={pending} onclick={apply}>Import {preview.count} reviewed records</Button>{/if}
    {#if results.length}<ul role="status" class="max-h-72 overflow-y-auto text-sm">{#each results as result (result.key)}<li class={result.imported ? "text-muted-foreground" : "text-destructive"}>{result.key}: {result.imported ? "Imported (or already present)" : result.error}</li>{/each}</ul>{/if}
    {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
  </Field.Group>
</SettingsSection>
