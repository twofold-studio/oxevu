<script lang="ts">
  import type { Show } from "$lib/media/grouping";
  import { href } from "$lib/state/router.svelte";
  import { plural } from "$lib/media/format";
  import { resumePosition, watchedFraction } from "$lib/media/progress";
  import { fileAvailable } from "$lib/state/catalog.svelte";
  import TitlePlate from "./TitlePlate.svelte";

  let { show }: { show: Show } = $props();

  const episodes = $derived(show.seasons.flatMap((season) => season.episodes));
  const watchedCount = $derived(episodes.filter((episode) => episode.completed).length);
  const progress = $derived(episodes.reduce((total, episode) => total + (watchedFraction(episode) ?? 0), 0));
  const allRemoved = $derived(show.seasons.every((s) => s.episodes.every((e) => !fileAvailable(e))));
  const activeDownload = $derived(
    show.seasons
      .flatMap((s) => s.episodes)
      .find((e) => e.download && !fileAvailable(e))
      ?.download ?? null,
  );
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
  detail={[show.next.show_year ?? show.next.year, facts.join(" · "), detail].filter(Boolean).join(" · ")}
  progress={progress > 0 ? progress / show.episodeCount : null}
  inProgress={watchedCount > 0 || episodes.some((episode) => resumePosition(episode) !== null)}
  watched={show.episodeCount > 0 && watchedCount === show.episodeCount}
  fileRemoved={allRemoved}
  download={activeDownload}
  needsIdentification={show.seasons.some((s) => s.episodes.some((e) => e.metadata_status === "needs_identification"))}
/>
