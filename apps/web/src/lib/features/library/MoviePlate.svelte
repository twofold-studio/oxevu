<script lang="ts">
  import type { Item } from "$lib/api/types";
  import { href } from "$lib/state/router.svelte";
  import { formatCodec, formatContainer, formatRuntime } from "$lib/media/format";
  import { watchedFraction } from "$lib/media/progress";
  import { fileAvailable } from "$lib/state/catalog.svelte";
  import TitlePlate from "./TitlePlate.svelte";

  let { item }: { item: Item } = $props();

  const facts = $derived(
    [formatContainer(item.container), formatCodec(item.video_codec)].filter((v): v is string => v !== null),
  );
</script>

<TitlePlate
  href={href.item(item.id)}
  title={item.title}
  poster={item.poster_url}
  {facts}
  detail={formatRuntime(item.duration_seconds)}
  progress={watchedFraction(item)}
  watched={item.completed}
  fileRemoved={!fileAvailable(item)}
  download={item.download}
  needsIdentification={item.metadata_status === "needs_identification"}
/>
