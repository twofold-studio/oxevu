<script lang="ts">
  import LogOutIcon from "@lucide/svelte/icons/log-out";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { buttonVariants } from "$lib/components/ui/button";
  import { session } from "$lib/state/session.svelte";
  import { catalog } from "$lib/state/catalog.svelte";
  import { href, router } from "$lib/state/router.svelte";
  import { errorMessage } from "$lib/api/client";
  import { toast } from "svelte-sonner";
  import { isMobileApp, requestServerPicker } from "$lib/mobile/connection";

  const username = $derived(session.user?.username ?? "");
  const initial = $derived(username.slice(0, 1).toUpperCase() || "?");

  async function signOut() {
    try {
      await session.signOut();
    } catch (error) {
      toast.error(`Sign out didn't reach the server: ${errorMessage(error)}`);
    } finally {
      catalog.reset();
      router.go(href.home(), true);
    }
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class={buttonVariants({ variant: "ghost", size: "icon" })}
    aria-label={`Account: ${username}`}
  >
    {#if session.user?.avatar}
      <img src={session.user.avatar} alt="" class="size-7 rounded-full object-cover" />
    {:else}<span
      class="grid size-7 place-items-center rounded-full border bg-secondary text-xs font-semibold text-secondary-foreground"
      aria-hidden="true">{initial}</span
    >{/if}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end" class="min-w-52">
    <DropdownMenu.Group>
      <DropdownMenu.Label class="flex flex-col gap-0.5">
        <span class="text-xs font-normal text-muted-foreground">Signed in as</span>
        <span class="truncate">{username}</span>
        {#if session.user?.is_admin}<span class="text-xs font-normal text-muted-foreground">Administrator</span>{/if}
      </DropdownMenu.Label>
    </DropdownMenu.Group>
    <DropdownMenu.Separator />
    <DropdownMenu.Group>
      {#if isMobileApp}
        <DropdownMenu.Item onSelect={requestServerPicker}>Change server</DropdownMenu.Item>
      {/if}
      <DropdownMenu.Item onSelect={() => router.go(href.settings())}>
        <SettingsIcon />
        Settings
      </DropdownMenu.Item>
      <DropdownMenu.Item onSelect={signOut}>
        <LogOutIcon />
        Sign out
      </DropdownMenu.Item>
    </DropdownMenu.Group>
  </DropdownMenu.Content>
</DropdownMenu.Root>
