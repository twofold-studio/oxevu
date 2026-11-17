<script lang="ts">
  import type { Route } from "$lib/state/router.svelte";
  import HomePage from "$lib/features/home/HomePage.svelte";
  import MoviesPage from "$lib/features/library/MoviesPage.svelte";
  import ShowsPage from "$lib/features/library/ShowsPage.svelte";
  import ShowPage from "$lib/features/library/ShowPage.svelte";
  import SearchPage from "$lib/features/library/SearchPage.svelte";
  import ItemPage from "$lib/features/details/ItemPage.svelte";
  import SettingsPage from "$lib/features/settings/SettingsPage.svelte";
  import PartiesPage from "$lib/features/party/PartiesPage.svelte";
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
{:else if route.name === "settings"}
  <SettingsPage target={route.target} />
{:else if route.name === "parties"}
  <PartiesPage />
{:else if route.name === "discover"}
  {#key JSON.stringify(route)}{#await import("$lib/features/acquisition/DiscoverPage.svelte") then page}<page.default query={route.query} kind={route.kind} page={route.page} filters={route.filters} />{/await}{/key}
{:else if route.name === "discover-title"}
  {#key `${route.kind}:${route.id}`}{#await import("$lib/features/acquisition/DiscoverTitlePage.svelte") then page}<page.default kind={route.kind} id={route.id} />{/await}{/key}
{:else if route.name === "requests"}
  {#await import("$lib/features/acquisition/RequestsPage.svelte") then page}<page.default />{/await}
{:else if route.name === "wanted"}
  {#await import("$lib/features/acquisition/WantedPage.svelte") then page}<page.default />{/await}
{:else if route.name === "activity"}
  {#await import("$lib/features/acquisition/ActivityPage.svelte") then page}<page.default />{/await}
{:else}
  <NotFound />
{/if}
