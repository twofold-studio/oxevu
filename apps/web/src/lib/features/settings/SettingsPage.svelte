<script lang="ts">
  import { onMount } from "svelte";
  import { MediaQuery } from "svelte/reactivity";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import Page from "$lib/components/Page.svelte";
  import { cn } from "$lib/utils";
  import { session } from "$lib/state/session.svelte";
  import { href, router } from "$lib/state/router.svelte";
  import { resolveSettings, type SettingsTarget } from "$lib/state/settings-pages";
  import SettingsNavList from "./navigation/SettingsNavList.svelte";
  import SettingsForbidden from "./navigation/SettingsForbidden.svelte";
  import SettingsPageView from "./navigation/SettingsPageView.svelte";

  let { target }: { target: SettingsTarget } = $props();

  // Same breakpoint as the settings aside (Tailwind `md`).
  const wide = new MediaQuery("min-width: 48rem");
  const resolution = $derived(resolveSettings(target, session.user));
  /** Where the URL should point instead, if anywhere. Replacing keeps Back meaningful. */
  const redirect = $derived(
    resolution.kind === "redirect"
      ? resolution.page
      : resolution.kind === "index" && wide.current
        ? resolution.first
        : null,
  );

  // Navigation is the side effect here: it rewrites the URL, then the router
  // re-parses it into a concrete page.
  $effect(() => {
    if (redirect) router.go(href.settings(redirect), true);
  });

  onMount(() => {
    // Permissions may have changed since sign-in; navigation follows them.
    void session.refresh();
  });

  const mobileLink =
    "inline-flex h-10 items-center gap-1.5 rounded-lg pr-2 text-sm text-muted-foreground transition-colors hover:text-foreground focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:outline-none";
</script>

<Page class="max-w-4xl">
  {#if resolution.kind === "index" && !redirect}
    <!-- Phones: settings open on this list; each page links back to it. -->
    <div class="flex flex-col gap-6">
      <a href={router.returnTo} class={mobileLink}><ArrowLeftIcon class="size-4" />Back to library</a>
      <h1 class="text-xl font-semibold tracking-tight">Settings</h1>
      <nav aria-label="Settings">
        <SettingsNavList variant="list" />
      </nav>
    </div>
  {:else if resolution.kind === "page" || resolution.kind === "forbidden"}
    <div class="flex flex-col gap-4">
      <a href={href.settings()} class={cn(mobileLink, "-mt-2 md:hidden")}><ChevronLeftIcon class="size-4" />All settings</a>
      {#key resolution.page}
        <div class="animate-rise">
          {#if resolution.kind === "page"}
            <SettingsPageView page={resolution.page} />
          {:else}
            <SettingsForbidden page={resolution.page} />
          {/if}
        </div>
      {/key}
    </div>
  {/if}
</Page>
