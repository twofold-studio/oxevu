<script lang="ts">
  import type { Route } from "$lib/state/router.svelte";
  import HomePage from "$lib/features/home/HomePage.svelte";
  import MoviesPage from "$lib/features/library/MoviesPage.svelte";
  import ShowsPage from "$lib/features/library/ShowsPage.svelte";
  import ShowPage from "$lib/features/library/ShowPage.svelte";
  import SearchPage from "$lib/features/library/SearchPage.svelte";
  import ItemPage from "$lib/features/details/ItemPage.svelte";
  import LibrariesPage from "$lib/features/settings/LibrariesPage.svelte";
  import NotFound from "$lib/components/NotFound.svelte";

  let { route }: { route: Route } = $props();
</script>

{#if route.name === "home"}
  <HomePage />
{:else if route.name === "movies"}
  <MoviesPage />
{:else if route.name === "shows"}
  <ShowsPage />
{:else if route.name === "show"}
  {#key route.ref}
    <ShowPage showRef={route.ref} season={route.season} />
  {/key}
{:else if route.name === "item"}
  {#key route.id}
    <ItemPage id={route.id} />
  {/key}
{:else if route.name === "search"}
  <SearchPage query={route.query} />
{:else if route.name === "libraries"}
  <LibrariesPage />
{:else}
  <NotFound />
{/if}
