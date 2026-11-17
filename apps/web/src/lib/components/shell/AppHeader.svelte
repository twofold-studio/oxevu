<script lang="ts">
  import { cn } from "$lib/utils";
  import { catalog } from "$lib/state/catalog.svelte";
  import { href, router } from "$lib/state/router.svelte";
  import { breadcrumbs } from "./breadcrumbs";
  import rustLogo from "../../../../../../assets/logo.png";
  import SearchBox from "./SearchBox.svelte";
  import AccountMenu from "./AccountMenu.svelte";
  import UsersRoundIcon from "@lucide/svelte/icons/users-round";
  import { Button } from "$lib/components/ui/button";
  import { session } from "$lib/state/session.svelte";
  import { canJoinParties } from "$lib/state/access";
  import WatchPartiesDialog from "$lib/features/party/WatchPartiesDialog.svelte";
  let partiesOpen = $state(false);

  const crumbs = $derived(breadcrumbs(router.route, catalog));
</script>

<header
  class="sticky top-0 z-40 flex h-16 shrink-0 items-center gap-3 border-b bg-background/95 px-4 backdrop-blur sm:px-5"
>
  <a href={href.home()} class="flex shrink-0 items-center gap-2 rounded-md font-semibold tracking-tight">
    <img src={rustLogo} alt="" class="size-7 shrink-0 rounded-md object-cover" />
    <span class="hidden sm:inline">Oxevu</span>
  </a>

  <nav aria-label="Breadcrumb" class="min-w-0 flex-1">
    <ol class="flex min-w-0 items-center gap-2 text-sm">
      {#each crumbs as crumb, index (index)}
        {@const last = index === crumbs.length - 1}
        <li class={cn("min-w-0 items-center gap-2", last ? "flex" : "hidden sm:flex")}>
          <span class="text-muted-foreground/60" aria-hidden="true">/</span>
          {#if crumb.href && !last}
            <a href={crumb.href} class="truncate text-muted-foreground transition-colors hover:text-foreground">
              {crumb.label}
            </a>
          {:else}
            <span class="truncate font-medium" aria-current={last ? "page" : undefined}>{crumb.label}</span>
          {/if}
        </li>
      {/each}
    </ol>
  </nav>

  <AccountMenu />
  {#if canJoinParties(session.user)}
    <Button
      onclick={() => (partiesOpen = true)}
      variant="ghost"
      size="icon"
      aria-label="Watch parties"
      title="Watch parties"
      aria-haspopup="dialog"
      aria-expanded={partiesOpen}
      class={router.route.name === "parties" || router.route.name === "party" ? "bg-accent text-foreground" : "text-muted-foreground"}
    ><UsersRoundIcon /></Button>
  {/if}
  <SearchBox class="hidden w-56 md:block xl:w-72" />
</header>

{#if partiesOpen}<WatchPartiesDialog bind:open={partiesOpen} />{/if}

<!-- Search stays reachable on phones without crowding the header. -->
<div class="border-b px-4 py-2 md:hidden">
  <SearchBox />
</div>
