<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { errorMessage } from "$lib/api/client";
  import { subtitlesApi } from "$lib/api/endpoints";
  import type { Subtitles } from "$lib/playback/subtitles.svelte";

  type Result = import("$lib/api/generated/types.gen").SubtitleResult;
  let { open = $bindable(false), subtitles, itemId, canDownload }: {
    open: boolean; subtitles: Subtitles; itemId: string; canDownload: boolean;
  } = $props();
  let language = $state("en");
  let results = $state<Result[] | null>(null);
  let busy = $state(false);
  let failure = $state<string | null>(null);

  async function search(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    if (!/^[a-z]{2,3}$/.test(language.trim())) {
      failure = "Enter a two- or three-letter language code, such as en or bul.";
      return;
    }
    busy = true;
    failure = null;
    results = null;
    try {
      const data = await subtitlesApi.search(itemId, language.trim());
      results = data.results;
    } catch (error) { failure = errorMessage(error); }
    finally { busy = false; }
  }

  async function download(result: Result) {
    if (busy || !canDownload) return;
    busy = true;
    failure = null;
    try {
      const { track } = await subtitlesApi.download(itemId, result.id, result.language);
      if (!(await subtitles.load(itemId, false))) {
        throw new Error(subtitles.error ?? "The subtitle downloaded, but the track list could not be refreshed.");
      }
      subtitles.select(track.id);
      open = false;
    } catch (error) { failure = errorMessage(error); }
    finally { busy = false; }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content portalProps={{ disabled: true }} size="wide">
    <Dialog.Header class="min-w-0 pr-6">
      <Dialog.Title>Find subtitles</Dialog.Title>
      <Dialog.Description class="min-w-0 break-words">Search OpenSubtitles for this title. Downloads use the configured account's quota and stay in Oxevu's data folder.</Dialog.Description>
    </Dialog.Header>
    <form class="min-w-0" onsubmit={search}>
      <Field.Group>
        <Field.Field>
          <Field.Label for="subtitle-language">Language code</Field.Label>
          <Input id="subtitle-language" bind:value={language} maxlength={3} placeholder="en" autocomplete="off" />
          <Field.Description>For example: en, bg, fr or de.</Field.Description>
        </Field.Field>
        <Button type="submit" disabled={busy}>{#if busy}<Spinner data-icon="inline-start" />{/if}Search</Button>
      </Field.Group>
    </form>
    {#if failure}<p class="text-sm text-destructive" role="alert">{failure}</p>{/if}
    {#if results}
      {#if results.length === 0}<p class="text-sm text-muted-foreground">No subtitles found for this language.</p>{/if}
      <ul class="flex min-w-0 flex-col divide-y rounded-md border">
        {#each results as result (result.id)}
          <li class="flex min-w-0 flex-col items-start gap-3 p-3 sm:flex-row sm:items-center">
            <div class="min-w-0 flex-1">
              <p class="text-sm [overflow-wrap:anywhere]">{result.label}</p>
              <p class="text-xs text-muted-foreground">{result.language}{result.hearing_impaired ? " · Hearing impaired" : ""} · {result.download_count} downloads</p>
            </div>
            <Button class="shrink-0" size="sm" variant="outline" disabled={busy || !canDownload} onclick={() => void download(result)}>Download</Button>
          </li>
        {/each}
      </ul>
    {/if}
    {#if !canDownload}<p class="text-xs text-muted-foreground">Your account needs conversion permission to download and prepare subtitles.</p>{/if}
  </Dialog.Content>
</Dialog.Root>
