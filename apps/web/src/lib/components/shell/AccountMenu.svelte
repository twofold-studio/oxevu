<script lang="ts">
  import LogOutIcon from "@lucide/svelte/icons/log-out";
  import FolderCogIcon from "@lucide/svelte/icons/folder-cog";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { buttonVariants } from "$lib/components/ui/button";
  import { session } from "$lib/state/session.svelte";
  import { catalog } from "$lib/state/catalog.svelte";
  import { href, router } from "$lib/state/router.svelte";
  import { errorMessage } from "$lib/api/client";
  import { toast } from "svelte-sonner";

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
    <span
      class="grid size-7 place-items-center rounded-full border bg-secondary text-xs font-semibold text-secondary-foreground"
      aria-hidden="true">{initial}</span
    >
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end" class="min-w-52">
    <DropdownMenu.Group>
      <DropdownMenu.Label class="flex flex-col gap-0.5">
        <span class="text-xs font-normal text-muted-foreground">Signed in as</span>
        <span class="truncate">{username}</span>
      </DropdownMenu.Label>
    </DropdownMenu.Group>
    <DropdownMenu.Separator />
    <DropdownMenu.Group>
      <DropdownMenu.Item onSelect={() => router.go(href.libraries())}>
        <FolderCogIcon />
        Libraries
      </DropdownMenu.Item>
      <DropdownMenu.Item onSelect={signOut}>
        <LogOutIcon />
        Sign out
      </DropdownMenu.Item>
    </DropdownMenu.Group>
  </DropdownMenu.Content>
</DropdownMenu.Root>
