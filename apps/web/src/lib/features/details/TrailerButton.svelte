<script lang="ts">
  import VideoIcon from "@lucide/svelte/icons/video";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { acquisitionApi } from "$lib/api/acquisition";
  import type { MediaKind } from "$lib/api/acquisition-types";

  let { kind, providerId, title }: {
    kind: MediaKind;
    providerId?: string | number | null;
    title: string;
  } = $props();

  let videoId = $state<string | null>(null);
  let open = $state(false);

  $effect(() => {
    const id = Number(providerId);
    const currentKind = kind;
    videoId = null;
    open = false;
    if (!Number.isSafeInteger(id) || id <= 0) return;
    const controller = new AbortController();
    void acquisitionApi.trailer(currentKind, id, controller.signal)
      .then(({ youtube_id }) => {
        if (!controller.signal.aborted && youtube_id && /^[A-Za-z0-9_-]{11}$/.test(youtube_id)) {
          videoId = youtube_id;
        }
      })
      .catch(() => { /* Optional metadata must not interrupt the title page. */ });
    return () => controller.abort();
  });
</script>

{#if videoId}
  <Dialog.Root bind:open>
    <Dialog.Trigger>
      {#snippet child({ props })}
        <Button {...props} variant="outline"><VideoIcon data-icon="inline-start" />Watch trailer</Button>
      {/snippet}
    </Dialog.Trigger>
    <Dialog.Content size="wide">
      <Dialog.Header>
        <Dialog.Title>{title} trailer</Dialog.Title>
        <Dialog.Description>Trailer provided by YouTube.</Dialog.Description>
      </Dialog.Header>
      {#if open}
        <iframe
          title={`${title} trailer`}
          src={`https://www.youtube-nocookie.com/embed/${videoId}?autoplay=1`}
          class="aspect-video w-full rounded-md border"
          referrerpolicy="strict-origin-when-cross-origin"
          allow="autoplay; encrypted-media; picture-in-picture"
          allowfullscreen
        ></iframe>
      {/if}
      <Dialog.Footer>
        <Button variant="outline" href={`https://www.youtube.com/watch?v=${videoId}`} target="_blank" rel="noopener noreferrer">
          Open on YouTube
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  </Dialog.Root>
{/if}
