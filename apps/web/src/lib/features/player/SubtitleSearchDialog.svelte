<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { request, errorMessage } from "$lib/api/client";
  import type { Subtitles, SubtitleTrack } from "$lib/playback/subtitles.svelte";

  interface Result { id: string; label: string; language: string; hearing_impaired: boolean; download_count: number }
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
      const data = await request<{ results: Result[] }>(`/items/${encodeURIComponent(itemId)}/subtitles/search?language=${encodeURIComponent(language.trim())}`);
      results = data.results;
    } catch (error) { failure = errorMessage(error); }
    finally { busy = false; }
  }

  async function download(result: Result) {
    if (busy || !canDownload) return;
    busy = true;
    failure = null;
    try {
      const { track } = await request<{ track: SubtitleTrack }>(`/items/${encodeURIComponent(itemId)}/subtitles/download`, {
        method: "POST", body: { id: result.id, language: result.language },
      });
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
    <Dialog.Header>
      <Dialog.Title>Find subtitles</Dialog.Title>
      <Dialog.Description>Search OpenSubtitles for this title. An administrator must configure the integration first. Downloads count against the account's quota and stay in Oxevu's data folder.</Dialog.Description>
    </Dialog.Header>
    <form onsubmit={search}>
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
      <ul class="flex flex-col divide-y rounded-md border">
        {#each results as result (result.id)}
          <li class="flex items-center gap-3 p-3">
            <div class="min-w-0 flex-1">
              <p class="text-sm break-words">{result.label}</p>
              <p class="text-xs text-muted-foreground">{result.language}{result.hearing_impaired ? " · Hearing impaired" : ""} · {result.download_count} downloads</p>
            </div>
            <Button size="sm" variant="outline" disabled={busy || !canDownload} onclick={() => void download(result)}>Download</Button>
          </li>
        {/each}
      </ul>
    {/if}
    {#if !canDownload}<p class="text-xs text-muted-foreground">Your account needs conversion permission to download and prepare subtitles.</p>{/if}
  </Dialog.Content>
</Dialog.Root>
