<script lang="ts">
  import type { Show } from "$lib/media/grouping";
  import { href } from "$lib/state/router.svelte";
  import { plural } from "$lib/media/format";
  import { fileAvailable } from "$lib/state/catalog.svelte";
  import TitlePlate from "./TitlePlate.svelte";

  let { show }: { show: Show } = $props();

  const watchedCount = $derived(show.seasons.reduce((n, s) => n + s.episodes.filter((e) => e.completed).length, 0));
  const allRemoved = $derived(show.seasons.every((s) => s.episodes.every((e) => !fileAvailable(e))));
  const facts = $derived([
    plural(show.seasons.length, "season"),
    plural(show.episodeCount, "episode"),
  ]);
  const detail = $derived(
    watchedCount === 0
      ? null
      : watchedCount === show.episodeCount
        ? "All episodes watched"
        : `${watchedCount} of ${show.episodeCount} watched`,
  );
</script>

<TitlePlate
  href={href.show(show.slug)}
  title={show.title}
  poster={show.posterUrl}
  {facts}
  {detail}
  progress={watchedCount > 0 ? watchedCount / show.episodeCount : null}
  watched={watchedCount === show.episodeCount}
  fileRemoved={allRemoved}
/>
