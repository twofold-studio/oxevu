<script lang="ts">
  import * as Field from "$lib/components/ui/field";
  import * as Select from "$lib/components/ui/select";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import type { CatalogEpisode, RequestScope, DiscoverTitle } from "$lib/api/acquisition-types";
  let { scope = $bindable(), title, episodes, monitoring = false }: { scope: RequestScope; title: DiscoverTitle; episodes: CatalogEpisode[]; monitoring?: boolean } = $props();
  const modes = $derived([
    ...(monitoring ? [{ value: "requests", label: "Approved requests" }] : []),
    { value: "all", label: "Whole series" }, { value: "existing", label: "Existing episodes" },
    { value: "future", label: "Future episodes" }, { value: "seasons", label: "Selected seasons" }, { value: "episodes", label: "Selected episodes" },
  ]);
  const seasons = $derived([...new Set(episodes.map((episode) => episode.season_number))].sort((a,b) => a-b));
  function seasonToggle(season: number, checked: boolean) { scope.seasons = checked ? [...scope.seasons, season] : scope.seasons.filter((n) => n !== season); }
  function episodeToggle(season: number, episode: number, checked: boolean) { scope.episodes = checked ? [...scope.episodes, { season, episode }] : scope.episodes.filter((e) => e.season !== season || e.episode !== episode); }
</script>

{#if title.kind === "tv"}
  <Field.Group>
    <Field.Field>
      <Field.Label for="request-scope">What to {monitoring ? "monitor" : "request"}</Field.Label>
      <Select.Root type="single" items={modes} bind:value={() => scope.mode, (v) => scope.mode = v as RequestScope["mode"]}>
        <Select.Trigger id="request-scope" class="w-full"><Select.Value placeholder="Choose episodes" /></Select.Trigger>
        <Select.Content><Select.Group>{#each modes as mode (mode.value)}<Select.Item value={mode.value}>{mode.label}</Select.Item>{/each}</Select.Group></Select.Content>
      </Select.Root>
      <Field.Description>Whole series includes future seasons. Specials are included only when selected.</Field.Description>
    </Field.Field>
    {#if scope.mode === "seasons"}
      <Field.Set><Field.Legend variant="label">Seasons</Field.Legend><Field.Group>
        {#each seasons as season (season)}
          <Field.Field orientation="horizontal"><Checkbox id={`scope-season-${season}`} checked={scope.seasons.includes(season)} onCheckedChange={(checked) => seasonToggle(season, checked === true)} /><Field.Label for={`scope-season-${season}`}>{season === 0 ? "Specials" : `Season ${season}`}</Field.Label></Field.Field>
        {/each}
      </Field.Group></Field.Set>
    {:else if scope.mode === "episodes"}
      <Field.Set><Field.Legend variant="label">Episodes</Field.Legend><Field.Group>
        {#each episodes as episode (episode.id)}
          <Field.Field orientation="horizontal">
            <Checkbox id={`scope-episode-${episode.id}`} checked={scope.episodes.some((e) => e.season === episode.season_number && e.episode === episode.episode_number)} onCheckedChange={(checked) => episodeToggle(episode.season_number, episode.episode_number, checked === true)} />
            <Field.Content><Field.Label for={`scope-episode-${episode.id}`}>Season {episode.season_number}, Episode {episode.episode_number} · {episode.name}</Field.Label><Field.Description>{episode.available ? "In your library" : episode.air_date ?? "Air date unknown"}</Field.Description></Field.Content>
          </Field.Field>
        {/each}
      </Field.Group></Field.Set>
    {/if}
  </Field.Group>
{/if}
